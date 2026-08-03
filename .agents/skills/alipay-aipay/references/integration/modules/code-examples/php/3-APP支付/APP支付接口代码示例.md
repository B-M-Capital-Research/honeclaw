# APP支付接口 - PHP 示例

## 请求示例

```php
<?php
require_once '../aop/AopClient.php';
require_once '../aop/AopCertClient.php';
require_once '../aop/AopCertification.php';
require_once '../aop/AlipayConfig.php';
require_once '../aop/request/AlipayTradeAppPayRequest.php';

// 初始化SDK
$alipayClient = new AopClient(getAlipayConfig());
// 构造请求参数以调用接口
$request = new AlipayTradeAppPayRequest();
$model = array();

// 设置商户订单号
$model['out_trade_no'] = "MERCHANT_ORDER_NO_PLACEHOLDER";

// 设置订单总金额
$model['total_amount'] = "9.00";

// 设置订单标题
$model['subject'] = "大乐透";

// 设置产品码
$model['product_code'] = "QUICK_MSECURITY_PAY";

// 设置订单包含的商品列表信息
$goodsDetail = array();
$goodsDetail0 = array();
$goodsDetail0['goods_name'] = "ipad";
$goodsDetail0['alipay_goods_id'] = "20010001";
$goodsDetail0['quantity'] = 1;
$goodsDetail0['price'] = "2000";
$goodsDetail0['goods_id'] = "apple-01";
$goodsDetail0['goods_category'] = "34543238";
$goodsDetail0['categories_tree'] = "124868003|126232002|126252004";
$goodsDetail0['show_url'] = "http://www.alipay.com/xxx.jpg";
$goodsDetail[] = $goodsDetail0;
$model['goods_detail'] = $goodsDetail;

// 设置订单绝对超时时间
$model['time_expire'] = "2016-12-31 10:05:00";

// 设置业务扩展参数
$extendParams = array();
$extendParams['sys_service_provider_id'] = "<SYS_SERVICE_PROVIDER_ID>";
$extendParams['hb_fq_seller_percent'] = "100";
$extendParams['hb_fq_num'] = "3";
$extendParams['industry_reflux_info'] = "{\"scene_code\":\"metro_tradeorder\",\"channel\":\"xxxx\",\"scene_data\":{\"asset_name\":\"ALIPAY\"}}";
$extendParams['royalty_freeze'] = "true";
$extendParams['card_type'] = "S0JP0000";
$model['extend_params'] = $extendParams;

// 设置公用回传参数
$model['passback_params'] = "merchantBizType%3d3C%26merchantBizNo%3dMERCHANT_BIZ_NO_PLACEHOLDER";

// 设置商户的原始订单号
$model['merchant_order_no'] = "MERCHANT_ORDER_NO_PLACEHOLDER";

// 设置外部指定买家
$extUserInfo = array();
$extUserInfo['cert_type'] = "IDENTITY_CARD";
$extUserInfo['cert_no'] = "<CERT_NO>";
$extUserInfo['mobile'] = "<MOBILE>";
$extUserInfo['name'] = "李明";
$extUserInfo['min_age'] = "18";
$extUserInfo['need_check_info'] = "F";
$extUserInfo['identity_hash'] = "<IDENTITY_HASH>";
$model['ext_user_info'] = $extUserInfo;

// 设置通知参数选项
$queryOptions = array();
$queryOptions[] = "hyb_amount";
$queryOptions[] = "enterprise_pay_info";
$model['query_options'] = $queryOptions;

$request->setBizContent(json_encode($model,JSON_UNESCAPED_UNICODE));
// 如果是第三方代调用模式，请设置app_auth_token（应用授权令牌）
$orderStr = $alipayClient->sdkExecute($request, "<-- 请填写应用授权令牌 -->");
echo $orderStr;

function getAlipayConfig()
{
    $privateKey  = '<-- 请填写您的应用私钥，例如：MIIE...（脱敏示例） -->';
    $alipayPublicKey = '<-- 请填写您的支付宝公钥，例如：MIIB...（脱敏示例） -->';
    $alipayConfig = new AlipayConfig();
    $alipayConfig->setServerUrl('https://openapi.alipay.com/gateway.do');
    $alipayConfig->setAppId('<-- 请填写您的AppId，例如：20190xxxxxx -->');
    $alipayConfig->setPrivateKey($privateKey);
    $alipayConfig->setFormat('json');
    $alipayConfig->setAlipayPublicKey($alipayPublicKey);
    $alipayConfig->setCharset('UTF-8');
    $alipayConfig->setSignType('RSA2');
    return $alipayConfig;
}
```

## 响应示例
### 正常示例
```
app_id=20190xxxxxx&biz_content=%7B%22time_expire%22%3A%222016-12-31+10%3A05%3A00%22%2C%22extend_params%22%3A%22%22%2C%22query_options%22%3A%22%5B%5C%22hyb_amount%5C%22%2C%5C%22enterprise_pay_info%5C%22%5D%22%2C%22subject%22%3A%22%E5%A4%A7%E4%B9%90%E9%80%8F%22%2C%22product_code%22%3A%22QUICK_MSECURITY_PAY%22%2C%22body%22%3A%22Iphone6+16G%22%2C%22passback_params%22%3A%22merchantBizType%253d3C%2526merchantBizNo%253dMERCHANT_BIZ_NO_PLACEHOLDER%22%2C%22specified_channel%22%3A%22pcredit%22%2C%22goods_detail%22%3A%22%22%2C%22merchant_order_no%22%3A%22MERCHANT_ORDER_NO_PLACEHOLDER%22%2C%22enable_pay_channels%22%3A%22pcredit%2CmoneyFund%2CdebitCardExpress%22%2C%22out_trade_no%22%3A%22MERCHANT_ORDER_NO_PLACEHOLDER%22%2C%22ext_user_info%22%3A%22%22%2C%22total_amount%22%3A%229.00%22%2C%22timeout_express%22%3A%2290m%22%2C%22disable_pay_channels%22%3A%22pcredit%2CmoneyFund%2CdebitCardExpress%22%2C%22agreement_sign_params%22%3A%22%22%7D&charset=UTF-8&format=json&method=alipay.trade.app.pay&sign=SIGN_PLACEHOLDER&sign_type=RSA2&timestamp=2014-07-24+03%3A07%3A50&version=1.0
```

