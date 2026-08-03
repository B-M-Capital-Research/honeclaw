/**
 * A2M 智能收产品接入示例 - Node.js 版本
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
 * npm install express alipay-sdk
 */

const express = require('express');
const { AlipaySdk } = require('alipay-sdk');
const crypto = require('crypto');

const app = express();
const PORT = 3000;

// ==================== 配置信息（实际使用时请从受保护的配置读取）====================
const CONFIG = {
  // 支付宝配置
  alipay: {
    appId: '<APP_ID>', // 运行时映射沙箱 appId
    privateKey: '<APP_PRIVATE_KEY>', // 运行时映射 appPrivatePkcsKey（PKCS#1）
    alipayPublicKey: '<ALIPAY_PUBLIC_KEY>', // 运行时映射 alipayPublicKey
    gateway: 'https://openapi.alipay.com/gateway.do',
    sellerId: '<SELLER_ID_2088>', // 商户 ID（2088 格式）
    serviceId: 'api_mock_service_id', // 仅用于沙箱联调；上线前替换为服务市场真实 serviceId
    merchantPrivateKey: '<APP_PRIVATE_KEY>' // 与 privateKey 使用同一运行时配置，用于商家签名
  },
  // 资源服务配置
  resource: {
    path: '/demo/a2m/resource',
    goodsName: 'AI 生成内容服务'
  }
};

// 初始化支付宝 SDK
const alipaySdk = new AlipaySdk({
  appId: CONFIG.alipay.appId,
  privateKey: CONFIG.alipay.privateKey,
  alipayPublicKey: CONFIG.alipay.alipayPublicKey,
  gateway: CONFIG.alipay.gateway,
  timeout: 30000,
});

// ==================== 工具方法 ====================

/**
 * 格式化支付宝时间戳：yyyy-MM-dd HH:mm:ss
 */
function formatAlipayTimestamp(date = new Date()) {
  const pad = (n) => n.toString().padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

/**
 * 生成商家签名（seller_signature）
 * @param {Object} params - 待签名参数
 * @param {String} privateKey - 商户私钥
 * @returns {String} Base64 编码的签名
 */
function generateSellerSignature(params, privateKey) {
  // 1. 按 key 字典序排序
  const keys = Object.keys(params).sort();
  
  // 2. 拼接签名内容
  const signContent = keys
    .filter(key => params[key] !== null && params[key] !== '')
    .map((key, index) => {
      const value = params[key];
      return `${key}=${value}`;
    })
    .join('&');
  
  // 3. 将裸 PKCS#1 Base64 临时解析为 DER 密钥对象，不修改原始配置。
  const sellerPrivateKey = crypto.createPrivateKey({
    key: Buffer.from(privateKey, 'base64'),
    format: 'der',
    type: 'pkcs1'
  });
  const sign = crypto
    .createSign('RSA-SHA256')
    .update(signContent, 'utf8')
    .sign(sellerPrivateKey, 'base64');
  
  return sign;
}

/**
 * 格式化日期为 ISO 8601 带时区偏移量格式（如 2026-05-15T12:08:36+08:00）
 * @param {Date} date - 日期对象
 * @returns {String} ISO 8601 带时区偏移量字符串
 */
function formatISO8601WithTimezone(date) {
  const pad = (n) => n.toString().padStart(2, '0');
  const year = date.getFullYear();
  const month = pad(date.getMonth() + 1);
  const day = pad(date.getDate());
  const hours = pad(date.getHours());
  const minutes = pad(date.getMinutes());
  const seconds = pad(date.getSeconds());
  const offset = -date.getTimezoneOffset();
  const offsetHours = pad(Math.floor(Math.abs(offset) / 60));
  const offsetMinutes = pad(Math.abs(offset) % 60);
  const offsetSign = offset >= 0 ? '+' : '-';
  return `${year}-${month}-${day}T${hours}:${minutes}:${seconds}${offsetSign}${offsetHours}:${offsetMinutes}`;
}

/**
 * Base64URL 编码
 */
function base64UrlEncode(str) {
  return Buffer.from(str, 'utf8')
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

/**
 * Base64URL 解码
 */
function base64UrlDecode(str) {
  // 补充 padding
  let padding = '=';
  while (str.length % 4) {
    str += padding;
  }
  str = str.replace(/-/g, '+').replace(/_/g, '/');
  return Buffer.from(str, 'base64').toString('utf8');
}

// ==================== 智能收产品接入示例接口 ====================

/**
 * 智能收产品统一接口
 * 
 * 核心协议流程演示：
 * 1. 不带 Payment-Proof Header：返回 HTTP 402 + Payment-Needed Header
 * 2. 带 Payment-Proof Header：验证支付 → 自动履约 → 返回资源
 * 
 * @route GET /demo/a2m/resource
 * @param {String} Payment-Proof - 支付凭证（从 Header 获取，可选）
 * @returns {Object} 未支付时返回 402，已支付时返回资源内容
 */
app.get(CONFIG.resource.path, async (req, res) => {
  const paymentProof = req.headers['payment-proof'];
  
  // 场景 1：用户未支付，返回 402 + Payment-Needed Header
  if (!paymentProof || paymentProof.trim() === '') {
    return createPaymentRequiredResponse(req, res);
  }
  
  // 场景 2：用户已支付，验证 Payment-Proof 并返回资源
  return verifyPaymentAndDeliverResource(req, res, paymentProof);
});

/**
 * 创建 402 支付请求响应
 */
async function createPaymentRequiredResponse(req, res) {
  try {
    // 1. 构造订单信息
    const outTradeNo = `ORDER_${Date.now()}`;
    const amount = '0.01'; // 单位：元
    const currency = 'CNY';
    const resourceId = CONFIG.resource.path;
    const goodsName = CONFIG.resource.goodsName;
    
    // 2. 计算支付截止时间（30 分钟后），使用带时区偏移量的 ISO 8601 格式
    const payBefore = new Date(Date.now() + 30 * 60 * 1000);
    const payBeforeStr = formatISO8601WithTimezone(payBefore);
    
    // 3. 生成商家签名
    const sellerSignature = generateSellerSignature({
      amount,
      currency,
      goods_name: goodsName,
      out_trade_no: outTradeNo,
      pay_before: payBeforeStr,
      resource_id: resourceId,
      seller_id: CONFIG.alipay.sellerId,
      service_id: CONFIG.alipay.serviceId
    }, CONFIG.alipay.merchantPrivateKey);
    
    // 4. 构造 Payment-Needed Header 内容
    const paymentNeeded = {
      protocol: {
        out_trade_no: outTradeNo,
        amount,
        currency,
        resource_id: resourceId,
        pay_before: payBeforeStr,
        seller_signature: sellerSignature,
        seller_sign_type: 'RSA2',
        seller_unique_id: CONFIG.alipay.sellerId
      },
      method: {
        seller_name: '测试商户',
        seller_id: CONFIG.alipay.sellerId,
        seller_app_id: CONFIG.alipay.appId,
        goods_name: goodsName,
        seller_unique_id_key: 'seller_id',
        service_id: CONFIG.alipay.serviceId
      }
    };
    
    // 5. Base64URL 编码
    const paymentNeededEncoded = base64UrlEncode(JSON.stringify(paymentNeeded));
    
    // 6. 构造 402 响应
    res.set('Payment-Needed', paymentNeededEncoded);
    res.status(402).json({
      code: 'Payment-Needed',
      message: '需要支付',
      out_trade_no: outTradeNo,
      amount,
      currency,
      goods_name: goodsName
    });
    
    console.log(`创建支付订单成功：outTradeNo=${outTradeNo}, amount=${amount}`);
    
  } catch (error) {
    console.error('创建订单失败:', error.message);
    res.status(500).json({
      code: 'CREATE_ORDER_ERROR',
      message: '创建订单失败：' + error.message
    });
  }
}

/**
 * 验证支付凭证并交付资源
 */
async function verifyPaymentAndDeliverResource(req, res, paymentProof) {
  try {
    // 1. 从 Payment-Proof 中解析订单信息
    let paymentProofValue, tradeNo, clientSession;
    
    try {
      const decodedProof = base64UrlDecode(paymentProof);
      const proofJson = JSON.parse(decodedProof);
      
      // 从 protocol 层获取 payment_proof 和 trade_no
      if (proofJson.protocol) {
        paymentProofValue = proofJson.protocol.payment_proof;
        tradeNo = proofJson.protocol.trade_no;
      }
      
      // 从 method 层获取 client_session
      if (proofJson.method) {
        clientSession = proofJson.method.client_session;
      }
      
      // 校验必要字段
      if (!paymentProofValue || paymentProofValue.trim() === '') {
        return res.status(400).json({
          code: 'INVALID_PAYMENT_PROOF_FORMAT',
          message: 'Payment-Proof 格式错误：缺少 payment_proof'
        });
      }
      
      if (!tradeNo || tradeNo.trim() === '') {
        return res.status(400).json({
          code: 'INVALID_PAYMENT_PROOF_FORMAT',
          message: 'Payment-Proof 格式错误：缺少 trade_no'
        });
      }
      
    } catch (error) {
      console.error('Payment-Proof 解析失败:', error.message);
      return res.status(400).json({
        code: 'INVALID_PAYMENT_PROOF_FORMAT',
        message: 'Payment-Proof 格式错误：' + error.message
      });
    }
    
    // 2. 调用支付宝 API 验证支付凭证
    const verifyResponse = await alipaySdk.exec('alipay.aipay.agent.payment.verify', {
      bizContent: {
        payment_proof: paymentProofValue,
        trade_no: tradeNo,
        client_session: clientSession
      }
    });
    
    // 3. 验证失败，返回错误
    // SDK 可能返回嵌套响应，也可能直接返回业务字段。
    const nestedResponse = verifyResponse.alipay_aipay_agent_payment_verify_response;
    const responseData = nestedResponse || verifyResponse;

    if (responseData.code !== '10000' && responseData.code !== 10000) {
      const errorCode = responseData.sub_code || responseData.errorCode;
      const errorMessage = responseData.sub_msg || responseData.msg || '支付凭证验证失败';
      console.error('支付凭证验证失败:', errorMessage);
      return res.status(400).json({
        code: errorCode,
        message: errorMessage
      });
    }
    
    // 4. 验证成功，获取订单信息
    // 同时兼容 SDK 的 snake_case 与 camelCase 业务字段。
    const verifyTradeNo = responseData.trade_no || responseData.tradeNo || '';
    const verifyOutTradeNo = responseData.out_trade_no || responseData.outTradeNo || '';
    // 当前 demo 在沙箱字段为空时回退到固定资源；生产实现必须从本地订单读取并校验资源 ID。
    const resourceIdVerified = responseData.resource_id || responseData.resourceId || CONFIG.resource.path;
    const active = responseData.active;

    if (process.env.A2M_DEBUG === 'true') {
      console.log('验付响应字段:', {
        code: responseData.code,
        tradeNo: verifyTradeNo,
        outTradeNo: verifyOutTradeNo,
        resourceId: resourceIdVerified,
        active
      });
    }
    
    console.log(`支付凭证验证成功：tradeNo=${verifyTradeNo}, outTradeNo=${verifyOutTradeNo}`);
    
    // 5. 校验凭证有效性（active=true 表示凭证有效）
    if (active !== true) {
      console.error(`支付凭证无效或已过期：outTradeNo=${verifyOutTradeNo}`);
      return res.status(400).json({
        code: 'INVALID_PAYMENT_PROOF',
        message: '支付凭证无效或已过期'
      });
    }
    
    // 6. 【TODO】查询订单是否存在（以数据库为准）
    // const order = await orderRepository.findByOutTradeNo(verifyOutTradeNo);
    // if (!order) {
    //   return res.status(404).json({
    //     code: 'ORDER_NOT_FOUND',
    //     message: '订单不存在'
    //   });
    // }
    
    // 7. 【TODO】资源防串校验
    // if (resourceIdVerified !== order.resourceId) {
    //   return res.status(403).json({
    //     code: 'RESOURCE_ID_MISMATCH',
    //     message: '资源 ID 不匹配，可能存在资源串改风险'
    //   });
    // }
    
    // 8. 【TODO】履约防重放校验（数据库幂等控制）
    // if (order.fulfillStatus === 'FULFILLED') {
    //   return res.status(200).json({
    //     code: 'ALREADY_FULFILLED',
    //     message: '订单已履约，不重复提供',
    //     already_fulfilled: true
    //   });
    // }
    
    // 9. 执行业务逻辑，生成资源内容
    const serviceResult = generateServiceResource(resourceIdVerified);
    
    // 10. 【TODO】履约记录落库（用于审计/售后/对账）
    // await fulfillmentRecordRepository.save({ ... });
    
    // 11. 【TODO】保存待确认履约状态
    // 实际生产中建议先保存 serviceResult 和 PENDING_CONFIRM 状态。
    // fulfillment.confirm 成功后再标记 FULFILLED；如果确认失败，
    // 允许同一笔 Payment-Proof 重试确认，避免误返回成功。
    // await orderRepository.update(verifyOutTradeNo, { 
    //   orderStatus: 'PAID',
    //   fulfillStatus: 'PENDING_CONFIRM',
    //   tradeNo: verifyTradeNo,
    //   serviceResult
    // });
    
    // 12. 发送履约确认到支付宝，确认成功后才返回成功交付
    const fulfillmentTradeNo = verifyTradeNo || tradeNo;
    console.log(`资源已生成，准备发送履约确认：outTradeNo=${verifyOutTradeNo}, tradeNo=${fulfillmentTradeNo}`);

    const fulfillmentConfirmed = await sendFulfillmentConfirm(fulfillmentTradeNo);
    if (!fulfillmentConfirmed) {
      return res.status(502).json({
        code: 'FULFILLMENT_CONFIRM_FAILED',
        message: '资源已生成但履约确认失败，请稍后使用同一 Payment-Proof 重试'
      });
    }

    // 12.1 【TODO】履约确认成功后，再标记订单已履约
    // await orderRepository.update(verifyOutTradeNo, {
    //   fulfillStatus: 'FULFILLED',
    //   fulfilledAt: new Date()
    // });

    console.log(`履约确认成功：outTradeNo=${verifyOutTradeNo}, tradeNo=${fulfillmentTradeNo}`);
    
    // 13. 构造 Payment-Validation Header
    const paymentValidation = {
      trade_no: fulfillmentTradeNo,
      out_trade_no: verifyOutTradeNo,
      validated: true,
      resource_id: resourceIdVerified
    };
    
    const paymentValidationEncoded = base64UrlEncode(JSON.stringify(paymentValidation));
    res.set('Payment-Validation', paymentValidationEncoded);
    
    // 14. 返回资源内容
    res.json({
      resource_id: resourceIdVerified,
      content: serviceResult,
      trade_no: fulfillmentTradeNo,
      out_trade_no: verifyOutTradeNo,
      already_fulfilled: false,
      fulfillment_confirmed: true
    });
    
  } catch (error) {
    console.error('支付凭证验证异常:', error.message);
    res.status(500).json({
      code: 'VERIFY_FAILED',
      message: '支付凭证验证失败：' + error.message
    });
  }
}

/**
 * 生成服务资源内容
 */
function generateServiceResource(resourceId) {
  return JSON.stringify({
    status: 'success',
    service_type: 'AI_CONTENT_GENERATION',
    resource_id: resourceId,
    content: '这是 AI 生成的内容示例，可根据实际业务替换为任意数字服务内容',
    generated_at: formatISO8601WithTimezone(new Date())
  });
}

/**
 * 发送履约确认
 */
async function sendFulfillmentConfirm(tradeNo) {
  if (!tradeNo) {
    console.error('履约确认失败：tradeNo 为空');
    return false;
  }

  try {
    console.log(`开始发送履约确认：tradeNo=${tradeNo}`);
    
    const response = await alipaySdk.exec('alipay.aipay.agent.fulfillment.confirm', {
      bizContent: {
        trade_no: tradeNo
      }
    });
    
    // 注意：SDK 返回的响应可能是扁平结构，也可能嵌套在响应键下，需兼容两种情况
    const responseData = response.alipay_aipay_agent_fulfillment_confirm_response || response;
    if (responseData.code === '10000' || responseData.code === 10000) {
      console.log(`履约确认成功：tradeNo=${tradeNo}`);
      return true;
    } else {
      const errorCode = responseData.sub_code || responseData.errorCode;
      const errorMessage = responseData.sub_msg || responseData.msg || '履约确认失败';
      console.error(`履约确认失败：tradeNo=${tradeNo}, errorCode=${errorCode}, errorMsg=${errorMessage}`);
      return false;
    }
    
  } catch (error) {
    console.error(`履约确认异常：tradeNo=${tradeNo}, error=${error.message}`);
    return false;
  }
}

// ==================== 启动服务 ====================

if (require.main === module) {
  app.listen(PORT, () => {
    console.log(`A2M 按量付费服务已启动：http://localhost:${PORT}${CONFIG.resource.path}`);
    console.log('测试步骤：');
    console.log('1. 无 Payment-Proof Header: curl http://localhost:3000/demo/a2m/resource');
    console.log('2. 有 Payment-Proof Header: curl -H "Payment-Proof: <value>" http://localhost:3000/demo/a2m/resource');
  });
}

module.exports = app;
