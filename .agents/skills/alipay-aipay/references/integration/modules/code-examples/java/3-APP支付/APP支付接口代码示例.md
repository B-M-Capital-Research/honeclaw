# APP支付接口 - Java 示例

## 请求示例

```java
package com.java.sdk.demo;

import com.alipay.api.AlipayApiException;
import com.alipay.api.AlipayClient;
import com.alipay.api.DefaultAlipayClient;
import com.alipay.api.AlipayConfig;
import com.alipay.api.request.AlipayTradeAppPayRequest;
import com.alipay.api.domain.ExtUserInfo;
import com.alipay.api.domain.AlipayTradeAppPayModel;
import com.alipay.api.response.AlipayTradeAppPayResponse;
import com.alipay.api.domain.ExtendParams;
import com.alipay.api.domain.GoodsDetail;

import com.alipay.api.FileItem;
import java.util.Base64;
import java.util.ArrayList;
import java.util.List;

public class AlipayTradeAppPay {

    public static void main(String[] args) throws AlipayApiException {
        // 初始化SDK
        AlipayClient alipayClient = new DefaultAlipayClient(getAlipayConfig());

        // 构造请求参数以调用接口
        AlipayTradeAppPayRequest request = new AlipayTradeAppPayRequest();
        AlipayTradeAppPayModel model = new AlipayTradeAppPayModel();
        
        // 设置商户订单号
        model.setOutTradeNo("MERCHANT_ORDER_NO_PLACEHOLDER");
        
        // 设置订单总金额
        model.setTotalAmount("9.00");
        
        // 设置订单标题
        model.setSubject("大乐透");
        
        // 设置产品码
        model.setProductCode("QUICK_MSECURITY_PAY");
        
        // 设置订单包含的商品列表信息
        List<GoodsDetail> goodsDetail = new ArrayList<GoodsDetail>();
        GoodsDetail goodsDetail0 = new GoodsDetail();
        goodsDetail0.setGoodsName("ipad");
        goodsDetail0.setAlipayGoodsId("20010001");
        goodsDetail0.setQuantity(1L);
        goodsDetail0.setPrice("2000");
        goodsDetail0.setGoodsId("apple-01");
        goodsDetail0.setGoodsCategory("34543238");
        goodsDetail0.setCategoriesTree("124868003|126232002|126252004");
        goodsDetail0.setShowUrl("http://www.alipay.com/xxx.jpg");
        goodsDetail.add(goodsDetail0);
        model.setGoodsDetail(goodsDetail);
        
        // 设置订单绝对超时时间
        model.setTimeExpire("2016-12-31 10:05:00");
        
        // 设置业务扩展参数
        ExtendParams extendParams = new ExtendParams();
        extendParams.setSysServiceProviderId("<SYS_SERVICE_PROVIDER_ID>");
        extendParams.setHbFqSellerPercent("100");
        extendParams.setHbFqNum("3");
        extendParams.setIndustryRefluxInfo("{\"scene_code\":\"metro_tradeorder\",\"channel\":\"xxxx\",\"scene_data\":{\"asset_name\":\"ALIPAY\"}}");
        extendParams.setRoyaltyFreeze("true");
        extendParams.setCardType("S0JP0000");
        model.setExtendParams(extendParams);
        
        // 设置公用回传参数
        model.setPassbackParams("merchantBizType%3d3C%26merchantBizNo%3dMERCHANT_BIZ_NO_PLACEHOLDER");
        
        // 设置商户的原始订单号
        model.setMerchantOrderNo("MERCHANT_ORDER_NO_PLACEHOLDER");
        
        // 设置外部指定买家
        ExtUserInfo extUserInfo = new ExtUserInfo();
        extUserInfo.setCertType("IDENTITY_CARD");
        extUserInfo.setCertNo("<CERT_NO>");
        extUserInfo.setMobile("<MOBILE>");
        extUserInfo.setName("李明");
        extUserInfo.setMinAge("18");
        extUserInfo.setNeedCheckInfo("F");
        extUserInfo.setIdentityHash("<IDENTITY_HASH>");
        model.setExtUserInfo(extUserInfo);
        
        // 设置通知参数选项
        List<String> queryOptions = new ArrayList<String>();
        queryOptions.add("hyb_amount");
        queryOptions.add("enterprise_pay_info");
        model.setQueryOptions(queryOptions);
        
        request.setBizModel(model);
        // 第三方代调用模式下请设置app_auth_token
        // request.putOtherTextParam("app_auth_token", "<-- 请填写应用授权令牌 -->");

        AlipayTradeAppPayResponse response = alipayClient.sdkExecute(request);
        String orderStr = response.getBody();
        System.out.println(orderStr);

        if (response.isSuccess()) {
            System.out.println("调用成功");
        } else {
            System.out.println("调用失败");
            // SDK 版本是"4.38.0.ALL"及以上,可以参考下面的示例获取诊断链接
            // String diagnosisUrl = DiagnosisUtils.getDiagnosisUrl(response);
            // System.out.println(diagnosisUrl);
        }
    }

    private static AlipayConfig getAlipayConfig() {
        String privateKey  = "<-- 请填写您的应用私钥，例如：MIIE...（脱敏示例） -->";
        String alipayPublicKey = "<-- 请填写您的支付宝公钥，例如：MIIB...（脱敏示例） -->";
        AlipayConfig alipayConfig = new AlipayConfig();
        alipayConfig.setServerUrl("https://openapi.alipay.com/gateway.do");
        alipayConfig.setAppId("<-- 请填写您的AppId，例如：20190xxxxxx -->");
        alipayConfig.setPrivateKey(privateKey);
        alipayConfig.setFormat("json");
        alipayConfig.setAlipayPublicKey(alipayPublicKey);
        alipayConfig.setCharset("UTF-8");
        alipayConfig.setSignType("RSA2");
        return alipayConfig;
    }
}
```

## 响应示例
### 正常示例
```
app_id=20190xxxxxx&biz_content=%7B%22time_expire%22%3A%222016-12-31+10%3A05%3A00%22%2C%22extend_params%22%3A%22%22%2C%22query_options%22%3A%22%5B%5C%22hyb_amount%5C%22%2C%5C%22enterprise_pay_info%5C%22%5D%22%2C%22subject%22%3A%22%E5%A4%A7%E4%B9%90%E9%80%8F%22%2C%22product_code%22%3A%22QUICK_MSECURITY_PAY%22%2C%22body%22%3A%22Iphone6+16G%22%2C%22passback_params%22%3A%22merchantBizType%253d3C%2526merchantBizNo%253dMERCHANT_BIZ_NO_PLACEHOLDER%22%2C%22specified_channel%22%3A%22pcredit%22%2C%22goods_detail%22%3A%22%22%2C%22merchant_order_no%22%3A%22MERCHANT_ORDER_NO_PLACEHOLDER%22%2C%22enable_pay_channels%22%3A%22pcredit%2CmoneyFund%2CdebitCardExpress%22%2C%22out_trade_no%22%3A%22MERCHANT_ORDER_NO_PLACEHOLDER%22%2C%22ext_user_info%22%3A%22%22%2C%22total_amount%22%3A%229.00%22%2C%22timeout_express%22%3A%2290m%22%2C%22disable_pay_channels%22%3A%22pcredit%2CmoneyFund%2CdebitCardExpress%22%2C%22agreement_sign_params%22%3A%22%22%7D&charset=UTF-8&format=json&method=alipay.trade.app.pay&sign=SIGN_PLACEHOLDER&sign_type=RSA2&timestamp=2014-07-24+03%3A07%3A50&version=1.0
```

