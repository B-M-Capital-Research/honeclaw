<?php
/**
 * A2M 智能收产品接入示例 - PHP 版本
 *
 * 本示例展示 A2M 核心协议与 SDK 调用流程。
 * 订单持久化、本地订单匹配、金额一致性、资源防串、幂等履约和失败重试
 * 需要结合商户实际数据库与订单模型实现。在上述控制完成实现并通过 checklist 前，
 * 禁止将本文件原样用于生产或判定为生产就绪。
 * 
 * 本文件演示 A2M 核心协议调用流程：
 * 1. 返回 402 Payment-Needed Header
 * 2. 验证 Payment-Proof 支付凭证
 * 3. 发送履约回执确认
 * 4. 返回资源内容
 * 
 * 依赖安装：
 * composer require alipay/alipay-sdk
 */

// ==================== 配置信息（实际使用时请从配置中心读取）====================
define('ALIPAY_CONFIG', [
    'appId' => '<APP_ID>',
    'privateKey' => '<APP_PRIVATE_KEY>', // 请填写您的应用私钥（PKCS#1 格式）
    'alipayPublicKey' => '<ALIPAY_PUBLIC_KEY>', // 请填写您的支付宝公钥
    'gateway' => 'https://openapi.alipay.com/gateway.do',
    'sellerId' => '<SELLER_ID_2088>', // 商户 ID（2088 格式）
    'serviceId' => 'api_mock_service_id', // 仅用于沙箱联调；上线前替换为服务市场真实 serviceId
    'merchantPrivateKey' => '<APP_PRIVATE_KEY>' // 请填写您的应用私钥（用于商家签名）
]);

define('RESOURCE_CONFIG', [
    'path' => '/demo/a2m/resource',
    'goodsName' => 'AI 生成内容服务'
]);

/**
 * 初始化 SDK
 */
function initAlipayClient() {
    require_once 'aop/AopClient.php';
    require_once 'aop/AlipayConfig.php';
    
    $config = new AlipayConfig();
    $config->setServerUrl(ALIPAY_CONFIG['gateway']);
    $config->setAppId(ALIPAY_CONFIG['appId']);
    $config->setPrivateKey(ALIPAY_CONFIG['privateKey']);
    $config->setFormat('json');
    $config->setAlipayPublicKey(ALIPAY_CONFIG['alipayPublicKey']);
    $config->setCharset('UTF-8');
    $config->setSignType('RSA2');
    
    return new AopClient($config);
}

/**
 * 格式化支付宝时间戳：yyyy-MM-dd HH:mm:ss
 */
function formatAlipayTimestamp($timestamp = null) {
    if ($timestamp === null) {
        $timestamp = time();
    }
    return date('Y-m-d H:i:s', $timestamp);
}

/**
 * 生成商家签名（seller_signature）
 */
function generateSellerSignature($params, $privateKey) {
    // 1. 按 key 字典序排序
    ksort($params);
    
    // 2. 拼接签名内容
    $signContent = [];
    foreach ($params as $key => $value) {
        if ($value !== null && $value !== '') {
            $signContent[] = "{$key}={$value}";
        }
    }
    $signString = implode('&', $signContent);
    
    // 3. OpenSSL 需要 PEM；仅为本次签名临时包装裸 PKCS#1 Base64，不修改原始配置。
    $privateKeyPem = "-----BEGIN RSA PRIVATE KEY-----\n"
        . chunk_split($privateKey, 64, "\n")
        . "-----END RSA PRIVATE KEY-----";
    $privateKeyResource = openssl_pkey_get_private($privateKeyPem);
    if ($privateKeyResource === false) {
        throw new RuntimeException('无法解析 PKCS#1 应用私钥');
    }
    if (!openssl_sign($signString, $signature, $privateKeyResource, OPENSSL_ALGO_SHA256)) {
        throw new RuntimeException('seller_signature 生成失败');
    }
    
    return base64_encode($signature);
}

/**
 * Base64URL 编码
 */
function base64UrlEncode($data) {
    return rtrim(strtr(base64_encode($data), '+/', '-_'), '=');
}

/**
 * Base64URL 解码
 */
function base64UrlDecode($data) {
    $padding = strlen($data) % 4;
    if ($padding) {
        $data .= str_repeat('=', 4 - $padding);
    }
    return base64_decode(strtr($data, '-_', '+/'));
}

/**
 * 发送 JSON 响应
 */
function jsonResponse($data, $statusCode = 200, $headers = []) {
    http_response_code($statusCode);
    header('Content-Type: application/json; charset=utf-8');
    
    foreach ($headers as $key => $value) {
        header("{$key}: {$value}");
    }
    
    echo json_encode($data, JSON_UNESCAPED_UNICODE);
    exit;
}

// ==================== 智能收产品接入示例接口 ====================

/**
 * 智能收产品统一接口
 * 
 * 核心协议流程演示：
 * 1. 不带 Payment-Proof Header：返回 HTTP 402 + Payment-Needed Header
 * 2. 带 Payment-Proof Header：验证支付 → 自动履约 → 返回资源
 */
function handleResourceRequest() {
    // 获取 Payment-Proof Header
    $paymentProof = isset($_SERVER['HTTP_PAYMENT_PROOF']) ? $_SERVER['HTTP_PAYMENT_PROOF'] : null;
    
    // 场景 1：用户未支付，返回 402 + Payment-Needed Header
    if (empty($paymentProof) || trim($paymentProof) === '') {
        createPaymentRequiredResponse();
        return;
    }
    
    // 场景 2：用户已支付，验证 Payment-Proof 并返回资源
    verifyPaymentAndDeliverResource($paymentProof);
}

/**
 * 创建 402 支付请求响应
 */
function createPaymentRequiredResponse() {
    try {
        // 1. 构造订单信息
        $outTradeNo = 'ORDER_' . time() . mt_rand(1000, 9999);
        $amount = '0.01'; // 单位：元
        $currency = 'CNY';
        $resourceId = RESOURCE_CONFIG['path'];
        $goodsName = RESOURCE_CONFIG['goodsName'];
        
        // 2. 计算支付截止时间（30 分钟后）
        $payBefore = date('c', time() + 30 * 60); // ISO 8601 格式
        
        // 3. 生成商家签名
        $sellerSignature = generateSellerSignature([
            'amount' => $amount,
            'currency' => $currency,
            'goods_name' => $goodsName,
            'out_trade_no' => $outTradeNo,
            'pay_before' => $payBefore,
            'resource_id' => $resourceId,
            'seller_id' => ALIPAY_CONFIG['sellerId'],
            'service_id' => ALIPAY_CONFIG['serviceId']
        ], ALIPAY_CONFIG['merchantPrivateKey']);
        
        // 4. 构造 Payment-Needed Header 内容
        $paymentNeeded = [
            'protocol' => [
                'out_trade_no' => $outTradeNo,
                'amount' => $amount,
                'currency' => $currency,
                'resource_id' => $resourceId,
                'pay_before' => $payBefore,
                'seller_signature' => $sellerSignature,
                'seller_sign_type' => 'RSA2',
                'seller_unique_id' => ALIPAY_CONFIG['sellerId']
            ],
            'method' => [
                'seller_name' => '测试商户',
                'seller_id' => ALIPAY_CONFIG['sellerId'],
                'seller_app_id' => ALIPAY_CONFIG['appId'],
                'goods_name' => $goodsName,
                'seller_unique_id_key' => 'seller_id',
                'service_id' => ALIPAY_CONFIG['serviceId']
            ]
        ];
        
        // 5. Base64URL 编码
        $paymentNeededEncoded = base64UrlEncode(json_encode($paymentNeeded, JSON_UNESCAPED_UNICODE));
        
        // 6. 构造 402 响应
        $response = [
            'code' => 'Payment-Needed',
            'message' => '需要支付',
            'out_trade_no' => $outTradeNo,
            'amount' => $amount,
            'currency' => $currency,
            'goods_name' => $goodsName
        ];
        
        error_log("创建支付订单成功：outTradeNo={$outTradeNo}, amount={$amount}");
        
        jsonResponse($response, 402, ['Payment-Needed' => $paymentNeededEncoded]);
        
    } catch (Exception $e) {
        error_log('创建订单失败：' . $e->getMessage());
        jsonResponse([
            'code' => 'CREATE_ORDER_ERROR',
            'message' => '创建订单失败：' . $e->getMessage()
        ], 500);
    }
}

/**
 * 验证支付凭证并交付资源
 */
function verifyPaymentAndDeliverResource($paymentProof) {
    try {
        // 1. 从 Payment-Proof 中解析订单信息
        $decodedProof = base64UrlDecode($paymentProof);
        $proofJson = json_decode($decodedProof, true);
        
        if (json_last_error() !== JSON_ERROR_NONE) {
            throw new Exception('Payment-Proof 解析失败：' . json_last_error_msg());
        }
        
        // 从 protocol 层获取 payment_proof 和 trade_no
        $paymentProofValue = null;
        $tradeNo = null;
        $clientSession = null;
        
        if (isset($proofJson['protocol'])) {
            $paymentProofValue = $proofJson['protocol']['payment_proof'] ?? null;
            $tradeNo = $proofJson['protocol']['trade_no'] ?? null;
        }
        
        // 从 method 层获取 client_session
        if (isset($proofJson['method'])) {
            $clientSession = $proofJson['method']['client_session'] ?? null;
        }
        
        // 校验必要字段
        if (empty($paymentProofValue)) {
            jsonResponse([
                'code' => 'INVALID_PAYMENT_PROOF_FORMAT',
                'message' => 'Payment-Proof 格式错误：缺少 payment_proof'
            ], 400);
        }
        
        if (empty($tradeNo)) {
            jsonResponse([
                'code' => 'INVALID_PAYMENT_PROOF_FORMAT',
                'message' => 'Payment-Proof 格式错误：缺少 trade_no'
            ], 400);
        }
        
        // 2. 调用支付宝 API 验证支付凭证
        $alipayClient = initAlipayClient();
        $request = new \AlipayAipayAgentPaymentVerifyRequest();
        
        $model = [
            'payment_proof' => $paymentProofValue,
            'trade_no' => $tradeNo,
            'client_session' => $clientSession
        ];
        
        $request->setBizContent(json_encode($model, JSON_UNESCAPED_UNICODE));
        $responseResult = $alipayClient->execute($request);
        $responseApiName = str_replace('.', '_', $request->getApiMethodName()) . '_response';
        $verifyResponse = $responseResult->$responseApiName;
        
        // 3. 验证失败，返回错误
        if (empty($verifyResponse->code) || $verifyResponse->code !== '10000') {
            error_log('支付凭证验证失败：' . ($verifyResponse->sub_msg ?? 'Unknown error'));
            jsonResponse([
                'code' => $verifyResponse->sub_code ?? 'VERIFY_FAILED',
                'message' => $verifyResponse->sub_msg ?? '支付凭证验证失败'
            ], 400);
        }
        
        // 4. 验证成功，获取订单信息
        $verifyTradeNo = $verifyResponse->trade_no ?? $verifyResponse->tradeNo ?? '';
        $verifyOutTradeNo = $verifyResponse->out_trade_no ?? $verifyResponse->outTradeNo ?? '';
        $resourceId = $verifyResponse->resource_id ?? $verifyResponse->resourceId ?? '';
        // 当前 demo 在沙箱字段为空时回退到固定资源；生产实现必须从本地订单读取并校验资源 ID。
        $resourceIdVerified = !empty($resourceId) ? $resourceId : RESOURCE_CONFIG['path'];
        $active = $verifyResponse->active ?? null;
        
        error_log("支付凭证验证成功：tradeNo={$verifyTradeNo}, outTradeNo={$verifyOutTradeNo}");
        
        // 5. 校验凭证有效性（active=true 表示凭证有效）
        if ($active !== true) {
            error_log("支付凭证无效或已过期：outTradeNo={$verifyOutTradeNo}");
            jsonResponse([
                'code' => 'INVALID_PAYMENT_PROOF',
                'message' => '支付凭证无效或已过期'
            ], 400);
        }
        
        // 6. 【TODO】查询订单是否存在（以数据库为准）
        // $order = $orderRepository->findByOutTradeNo($verifyOutTradeNo);
        // if (!$order) {
        //     jsonResponse([
        //         'code' => 'ORDER_NOT_FOUND',
        //         'message' => '订单不存在'
        //     ], 404);
        // }
        
        // 7. 【TODO】资源防串校验
        // if ($resourceId !== $order->getResourceId()) {
        //     jsonResponse([
        //         'code' => 'RESOURCE_ID_MISMATCH',
        //         'message' => '资源 ID 不匹配，可能存在资源串改风险'
        //     ], 403);
        // }
        
        // 8. 【TODO】履约防重放校验（数据库幂等控制）
        // if ($order->getFulfillStatus() === 'FULFILLED') {
        //     jsonResponse([
        //         'code' => 'ALREADY_FULFILLED',
        //         'message' => '订单已履约，不重复提供',
        //         'already_fulfilled' => true
        //     ], 200);
        // }
        
        // 9. 执行业务逻辑，生成资源内容
        $serviceResult = generateServiceResource($resourceIdVerified);
        
        // 10. 【TODO】履约记录落库（用于审计/售后/对账）
        // $fulfillmentRecordRepository->save([...]);
        
        // 11. 【TODO】保存待确认履约状态
        // 实际生产中建议先保存 $serviceResult 和 PENDING_CONFIRM 状态。
        // fulfillment.confirm 成功后再标记 FULFILLED；如果确认失败，
        // 允许同一笔 Payment-Proof 重试确认，避免误返回成功。
        // $orderRepository->update($verifyOutTradeNo, [
        //     'orderStatus' => 'PAID',
        //     'fulfillStatus' => 'PENDING_CONFIRM',
        //     'tradeNo' => $verifyTradeNo,
        //     'serviceResult' => $serviceResult
        // ]);

        $fulfillmentTradeNo = $verifyTradeNo ?: $tradeNo;
        error_log("资源已生成，准备发送履约确认：outTradeNo={$verifyOutTradeNo}, tradeNo={$fulfillmentTradeNo}");

        // 12. 发送履约确认到支付宝，确认成功后才返回成功交付
        if (!sendFulfillmentConfirm($fulfillmentTradeNo)) {
            jsonResponse([
                'code' => 'FULFILLMENT_CONFIRM_FAILED',
                'message' => '资源已生成但履约确认失败，请稍后使用同一 Payment-Proof 重试'
            ], 502);
        }

        // 12.1 【TODO】履约确认成功后，再标记订单已履约
        // $orderRepository->update($verifyOutTradeNo, [
        //     'fulfillStatus' => 'FULFILLED',
        //     'fulfilledAt' => date('c')
        // ]);

        error_log("履约确认成功：outTradeNo={$verifyOutTradeNo}, tradeNo={$fulfillmentTradeNo}");
        
        // 13. 构造 Payment-Validation Header
        $paymentValidation = [
            'trade_no' => $fulfillmentTradeNo,
            'out_trade_no' => $verifyOutTradeNo,
            'validated' => true,
            'resource_id' => $resourceIdVerified
        ];
        
        $paymentValidationEncoded = base64UrlEncode(json_encode($paymentValidation, JSON_UNESCAPED_UNICODE));
        
        // 14. 返回资源内容
        jsonResponse([
            'resource_id' => $resourceIdVerified,
            'content' => $serviceResult,
            'trade_no' => $fulfillmentTradeNo,
            'out_trade_no' => $verifyOutTradeNo,
            'already_fulfilled' => false,
            'fulfillment_confirmed' => true
        ], 200, ['Payment-Validation' => $paymentValidationEncoded]);
        
    } catch (Exception $e) {
        error_log('支付凭证验证异常：' . $e->getMessage());
        jsonResponse([
            'code' => 'VERIFY_FAILED',
            'message' => '支付凭证验证失败：' . $e->getMessage()
        ], 500);
    }
}

/**
 * 生成服务资源内容
 */
function generateServiceResource($resourceId) {
    return json_encode([
        'status' => 'success',
        'service_type' => 'AI_CONTENT_GENERATION',
        'resource_id' => $resourceId,
        'content' => '这是 AI 生成的内容示例，可根据实际业务替换为任意数字服务内容',
        'generated_at' => date('c')
    ], JSON_UNESCAPED_UNICODE);
}

/**
 * 发送履约确认
 */
function sendFulfillmentConfirm($tradeNo) {
    if (empty($tradeNo)) {
        error_log('履约确认失败：tradeNo 为空');
        return false;
    }

    try {
        error_log("开始发送履约确认：tradeNo={$tradeNo}");
        
        $alipayClient = initAlipayClient();
        $request = new \AlipayAipayAgentFulfillmentConfirmRequest();
        
        $request->setBizContent(json_encode([
            'trade_no' => $tradeNo
        ], JSON_UNESCAPED_UNICODE));
        
        $responseResult = $alipayClient->execute($request);
        $responseApiName = str_replace('.', '_', $request->getApiMethodName()) . '_response';
        $response = $responseResult->$responseApiName;
        
        if (!empty($response->code) && $response->code === '10000') {
            error_log("履约确认成功：tradeNo={$tradeNo}");
            return true;
        } else {
            error_log("履约确认失败：tradeNo={$tradeNo}, errorCode=" . 
                ($response->sub_code ?? 'Unknown') . ", errorMsg=" . 
                ($response->sub_msg ?? 'Unknown'));
            return false;
        }
        
    } catch (Exception $e) {
        error_log("履约确认异常：tradeNo={$tradeNo}, error=" . $e->getMessage());
        return false;
    }
}

// ==================== 路由处理 ====================

// 简单路由
$requestUri = parse_url($_SERVER['REQUEST_URI'], PHP_URL_PATH);
if ($requestUri === RESOURCE_CONFIG['path'] && $_SERVER['REQUEST_METHOD'] === 'GET') {
    handleResourceRequest();
} else {
    http_response_code(404);
    echo json_encode(['code' => 'NOT_FOUND', 'message' => '接口不存在'], JSON_UNESCAPED_UNICODE);
}
