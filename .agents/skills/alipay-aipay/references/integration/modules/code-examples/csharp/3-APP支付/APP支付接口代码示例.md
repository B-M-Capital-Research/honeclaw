# APP支付接口 - C# 示例

## 请求示例

```csharp
using System;
using System.Collections.Generic;
using Aop.Api;
using Aop.Api.Request;
using Aop.Api.Response;
using Aop.Api.Domain;
using Aop.Api.Util;
namespace SdkDemoTest
{
    public class AlipayTradeAppPay
    {
        public static void Main(string[] args) 
        {
            // 初始化SDK
            IAopClient alipayClient = new DefaultAopClient(GetAlipayConfig());
            // 构造请求参数以调用接口
            AlipayTradeAppPayRequest request = new AlipayTradeAppPayRequest();
            AlipayTradeAppPayModel model = new AlipayTradeAppPayModel();
            
            // 设置商户订单号
            model.OutTradeNo = "MERCHANT_ORDER_NO_PLACEHOLDER";
            
            // 设置订单总金额
            model.TotalAmount = "9.00";
            
            // 设置订单标题
            model.Subject = "大乐透";
            
            // 设置产品码
            model.ProductCode = "QUICK_MSECURITY_PAY";
            
            // 设置订单包含的商品列表信息
            List<GoodsDetail> goodsDetail = new List<GoodsDetail>();
            GoodsDetail goodsDetail0 = new GoodsDetail();
            goodsDetail0.GoodsName = "ipad";
            goodsDetail0.AlipayGoodsId = "20010001";
            goodsDetail0.Quantity = 1;
            goodsDetail0.Price = "2000";
            goodsDetail0.GoodsId = "apple-01";
            goodsDetail0.GoodsCategory = "34543238";
            goodsDetail0.CategoriesTree = "124868003|126232002|126252004";
            goodsDetail0.ShowUrl = "http://www.alipay.com/xxx.jpg";
            goodsDetail.Add(goodsDetail0);
            model.GoodsDetail = goodsDetail;
            
            // 设置订单绝对超时时间
            model.TimeExpire = "2016-12-31 10:05:00";
            
            // 设置业务扩展参数
            ExtendParams extendParams = new ExtendParams();
            extendParams.SysServiceProviderId = "<SYS_SERVICE_PROVIDER_ID>";
            extendParams.HbFqSellerPercent = "100";
            extendParams.HbFqNum = "3";
            extendParams.IndustryRefluxInfo = "{\"scene_code\":\"metro_tradeorder\",\"channel\":\"xxxx\",\"scene_data\":{\"asset_name\":\"ALIPAY\"}}";
            extendParams.RoyaltyFreeze = "true";
            extendParams.CardType = "S0JP0000";
            model.ExtendParams = extendParams;
            
            // 设置公用回传参数
            model.PassbackParams = "merchantBizType%3d3C%26merchantBizNo%3dMERCHANT_BIZ_NO_PLACEHOLDER";
            
            // 设置商户的原始订单号
            model.MerchantOrderNo = "MERCHANT_ORDER_NO_PLACEHOLDER";
            
            // 设置外部指定买家
            ExtUserInfo extUserInfo = new ExtUserInfo();
            extUserInfo.CertType = "IDENTITY_CARD";
            extUserInfo.CertNo = "<CERT_NO>";
            extUserInfo.Mobile = "<MOBILE>";
            extUserInfo.Name = "李明";
            extUserInfo.MinAge = "18";
            extUserInfo.NeedCheckInfo = "F";
            extUserInfo.IdentityHash = "<IDENTITY_HASH>";
            model.ExtUserInfo = extUserInfo;
            
            // 设置通知参数选项
            List<String> queryOptions = new List<String>();
            queryOptions.Add("hyb_amount");
            queryOptions.Add("enterprise_pay_info");
            model.QueryOptions = queryOptions;
            
            request.SetBizModel(model);
            // 第三方代调用模式下请设置app_auth_token
            // request.PutOtherTextParam("app_auth_token", "<-- 请填写应用授权令牌 -->");

            AlipayTradeAppPayResponse response = alipayClient.SdkExecute(request);
            string orderStr = response.Body;
            Console.WriteLine(orderStr);

            if(!response.IsError)
            {
                Console.WriteLine("调用成功");
            }
            else
            {
                Console.WriteLine("调用失败");
            }
        }

        private static AlipayConfig GetAlipayConfig()
        {
            string privateKey  = "<-- 请填写您的应用私钥，例如：MIIE...（脱敏示例） -->";
            string alipayPublicKey = "<-- 请填写您的支付宝公钥，例如：MIIB...（脱敏示例） -->";
            AlipayConfig alipayConfig = new AlipayConfig();
            alipayConfig.ServerUrl = "https://openapi.alipay.com/gateway.do";
            alipayConfig.AppId = "<-- 请填写您的AppId，例如：20190xxxxxx -->";
            alipayConfig.PrivateKey = privateKey;
            alipayConfig.Format = "json";
            alipayConfig.AlipayPublicKey = alipayPublicKey;
            alipayConfig.Charset = "UTF-8";
            alipayConfig.SignType = "RSA2";
            return alipayConfig;
        }
    }
}
```

## 响应示例
### 正常示例
```
app_id=20190xxxxxx&biz_content=%7B%22time_expire%22%3A%222016-12-31+10%3A05%3A00%22%2C%22extend_params%22%3A%22%22%2C%22query_options%22%3A%22%5B%5C%22hyb_amount%5C%22%2C%5C%22enterprise_pay_info%5C%22%5D%22%2C%22subject%22%3A%22%E5%A4%A7%E4%B9%90%E9%80%8F%22%2C%22product_code%22%3A%22QUICK_MSECURITY_PAY%22%2C%22body%22%3A%22Iphone6+16G%22%2C%22passback_params%22%3A%22merchantBizType%253d3C%2526merchantBizNo%253dMERCHANT_BIZ_NO_PLACEHOLDER%22%2C%22specified_channel%22%3A%22pcredit%22%2C%22goods_detail%22%3A%22%22%2C%22merchant_order_no%22%3A%22MERCHANT_ORDER_NO_PLACEHOLDER%22%2C%22enable_pay_channels%22%3A%22pcredit%2CmoneyFund%2CdebitCardExpress%22%2C%22out_trade_no%22%3A%22MERCHANT_ORDER_NO_PLACEHOLDER%22%2C%22ext_user_info%22%3A%22%22%2C%22total_amount%22%3A%229.00%22%2C%22timeout_express%22%3A%2290m%22%2C%22disable_pay_channels%22%3A%22pcredit%2CmoneyFund%2CdebitCardExpress%22%2C%22agreement_sign_params%22%3A%22%22%7D&charset=UTF-8&format=json&method=alipay.trade.app.pay&sign=SIGN_PLACEHOLDER&sign_type=RSA2&timestamp=2014-07-24+03%3A07%3A50&version=1.0
```

