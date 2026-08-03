# APP支付接口 - Python 示例

## 请求示例

```python
import json
from alipay.aop.api.AlipayClientConfig import AlipayClientConfig
from alipay.aop.api.DefaultAlipayClient import DefaultAlipayClient
from alipay.aop.api.request.AlipayTradeAppPayRequest import AlipayTradeAppPayRequest
from alipay.aop.api.domain.AlipayTradeAppPayModel import AlipayTradeAppPayModel

# 初始化SDK
config = AlipayClientConfig()
config.server_url = "https://openapi.alipay.com/gateway.do"
config.app_id = "<-- 请填写您的AppId，例如：20190xxxxxx -->"
config.app_private_key = "<-- 请填写您的应用私钥，例如：MIIE...（脱敏示例） -->"
config.alipay_public_key = "<-- 请填写您的支付宝公钥，例如：MIIB...（脱敏示例） -->"
config.charset = "utf-8"
config.sign_type = "RSA2"
client = DefaultAlipayClient(alipay_client_config=config)

# 构造请求参数以调用接口
request = AlipayTradeAppPayRequest()
model = AlipayTradeAppPayModel()

model.out_trade_no = "MERCHANT_ORDER_NO_PLACEHOLDER"
model.total_amount = "9.00"
model.subject = "大乐透"
model.product_code = "QUICK_MSECURITY_PAY"

request.biz_model = model
# 第三方代调用模式下请设置app_auth_token
# request.add_other_text_param("app_auth_token", "<-- 请填写应用授权令牌 -->")

order_str = client.sdk_execute(request)
print(order_str)
```

## 响应示例
### 正常示例
```
app_id=20190xxxxxx&biz_content=%7B%22time_expire%22%3A%222016-12-31+10%3A05%3A00%22%2C%22extend_params%22%3A%22%22%2C%22query_options%22%3A%22%5B%5C%22hyb_amount%5C%22%2C%5C%22enterprise_pay_info%5C%22%5D%22%2C%22subject%22%3A%22%E5%A4%A7%E4%B9%90%E9%80%8F%22%2C%22product_code%22%3A%22QUICK_MSECURITY_PAY%22%2C%22body%22%3A%22Iphone6+16G%22%2C%22passback_params%22%3A%22merchantBizType%253d3C%2526merchantBizNo%253dMERCHANT_BIZ_NO_PLACEHOLDER%22%2C%22specified_channel%22%3A%22pcredit%22%2C%22goods_detail%22%3A%22%22%2C%22merchant_order_no%22%3A%22MERCHANT_ORDER_NO_PLACEHOLDER%22%2C%22enable_pay_channels%22%3A%22pcredit%2CmoneyFund%2CdebitCardExpress%22%2C%22out_trade_no%22%3A%22MERCHANT_ORDER_NO_PLACEHOLDER%22%2C%22ext_user_info%22%3A%22%22%2C%22total_amount%22%3A%229.00%22%2C%22timeout_express%22%3A%2290m%22%2C%22disable_pay_channels%22%3A%22pcredit%2CmoneyFund%2CdebitCardExpress%22%2C%22agreement_sign_params%22%3A%22%22%7D&charset=UTF-8&format=json&method=alipay.trade.app.pay&sign=SIGN_PLACEHOLDER&sign_type=RSA2&timestamp=2014-07-24+03%3A07%3A50&version=1.0
```
