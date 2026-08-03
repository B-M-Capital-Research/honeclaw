/*
 * A2M 智能收产品接入示例 - C# 版本
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
 * dotnet add package AlipaySDKNet.Standard
 * dotnet add package Microsoft.AspNetCore.App
 */

using System;
using System.Collections.Generic;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.DependencyInjection;
using Aop.Api;
using Aop.Api.Request;
using Aop.Api.Response;
using Aop.Api.Domain;

namespace A2MPaymentDemo
{
    class Program
    {
        // ==================== 配置信息（实际使用时请从配置中心读取）====================
        private static readonly AlipayConfig AlipayConfig = new AlipayConfig
        {
            ServerUrl = "https://openapi.alipay.com/gateway.do",
            AppId = "<APP_ID>",
            PrivateKey = "<APP_PRIVATE_KEY>", // 请填写您的应用私钥（PKCS#1 格式）
            AlipayPublicKey = "<ALIPAY_PUBLIC_KEY>", // 请填写您的支付宝公钥
            Format = "json",
            Charset = "UTF-8",
            SignType = "RSA2"
        };

        private static readonly string SellerId = "<SELLER_ID_2088>"; // 商户 ID（2088 格式）
        private static readonly string ServiceId = "api_mock_service_id"; // 仅用于沙箱联调；上线前替换为服务市场真实 serviceId
        private static readonly string MerchantPrivateKey = "<APP_PRIVATE_KEY>"; // 请填写您的应用私钥（用于商家签名）
        
        private const string ResourcePath = "/demo/a2m/resource";
        private const string GoodsName = "AI 生成内容服务";

        // 初始化支付宝客户端
        private static readonly IAopClient AlipayClient = new DefaultAopClient(AlipayConfig);

        static void Main(string[] args)
        {
            var builder = WebApplication.CreateBuilder(args);
            var app = builder.Build();

            // 路由处理
            app.MapGet(ResourcePath, async (HttpRequest request, HttpResponse response) =>
            {
                // 获取 Payment-Proof Header
                var paymentProof = request.Headers["Payment-Proof"].FirstOrDefault();

                // 场景 1：用户未支付，返回 402 + Payment-Needed Header
                if (string.IsNullOrWhiteSpace(paymentProof))
                {
                    await CreatePaymentRequiredResponse(response);
                    return;
                }

                // 场景 2：用户已支付，验证 Payment-Proof 并返回资源
                await VerifyPaymentAndDeliverResource(response, paymentProof);
            });

            Console.WriteLine($"A2M 按量付费服务已启动：http://localhost:5000{ResourcePath}");
            Console.WriteLine("测试步骤：");
            Console.WriteLine($"1. 无 Payment-Proof Header: curl http://localhost:5000{ResourcePath}");
            Console.WriteLine($"2. 有 Payment-Proof Header: curl -H \"Payment-Proof: <value>\" http://localhost:5000{ResourcePath}");

            app.Run();
        }

        // ==================== 工具方法 ====================

        /// <summary>
        /// 格式化支付宝时间戳：yyyy-MM-dd HH:mm:ss
        /// </summary>
        private static string FormatAlipayTimestamp(DateTime? dateTime = null)
        {
            return (dateTime ?? DateTime.Now).ToString("yyyy-MM-dd HH:mm:ss");
        }

        /// <summary>
        /// Base64URL 编码
        /// </summary>
        private static string Base64UrlEncode(string data)
        {
            var bytes = Encoding.UTF8.GetBytes(data);
            var base64 = Convert.ToBase64String(bytes);
            return base64.Replace('+', '-').Replace('/', '_').TrimEnd('=');
        }

        /// <summary>
        /// Base64URL 解码
        /// </summary>
        private static string Base64UrlDecode(string data)
        {
            // 补充 padding
            var padding = data.Length % 4;
            if (padding != 0)
            {
                data += new string('=', 4 - padding);
            }
            
            var base64 = data.Replace('-', '+').Replace('_', '/');
            var bytes = Convert.FromBase64String(base64);
            return Encoding.UTF8.GetString(bytes);
        }

        /// <summary>
        /// 生成商家签名（seller_signature）
        /// </summary>
        private static string GenerateSellerSignature(Dictionary<string, string> parameters, string privateKey)
        {
            // 1. 按 key 字典序排序
            var sortedKeys = parameters.Keys.OrderBy(k => k).ToList();
            
            // 2. 拼接签名内容
            var signContent = new StringBuilder();
            var first = true;
            foreach (var key in sortedKeys)
            {
                var value = parameters[key];
                if (!string.IsNullOrEmpty(value))
                {
                    if (!first)
                    {
                        signContent.Append("&");
                    }
                    signContent.Append($"{key}={value}");
                    first = false;
                }
            }

            // 3. RSA2 签名
            using (var rsa = RSA.Create())
            {
                rsa.ImportRSAPrivateKey(Convert.FromBase64String(privateKey), out _);
                var signature = rsa.SignData(Encoding.UTF8.GetBytes(signContent.ToString()), HashAlgorithmName.SHA256, RSASignaturePadding.Pkcs1);
                return Convert.ToBase64String(signature);
            }
        }

        // ==================== 智能收产品接入示例接口 ====================

        /// <summary>
        /// 创建 402 支付请求响应
        /// </summary>
        private static async Task CreatePaymentRequiredResponse(HttpResponse response)
        {
            try
            {
                // 1. 构造订单信息
                var outTradeNo = $"ORDER_{DateTimeOffset.Now.ToUnixTimeMilliseconds()}";
                var amount = "0.01";
                var currency = "CNY";
                var resourceId = ResourcePath;
                var goodsName = GoodsName;

                // 2. 计算支付截止时间（30 分钟后）
                var payBefore = DateTimeOffset.Now.AddMinutes(30).ToString("o");

                // 3. 生成商家签名
                var signParams = new Dictionary<string, string>
                {
                    ["amount"] = amount,
                    ["currency"] = currency,
                    ["goods_name"] = goodsName,
                    ["out_trade_no"] = outTradeNo,
                    ["pay_before"] = payBefore,
                    ["resource_id"] = resourceId,
                    ["seller_id"] = SellerId,
                    ["service_id"] = ServiceId
                };

                var sellerSignature = GenerateSellerSignature(signParams, MerchantPrivateKey);

                // 4. 构造 Payment-Needed Header 内容
                var paymentNeeded = new
                {
                    protocol = new
                    {
                        out_trade_no = outTradeNo,
                        amount = amount,
                        currency = currency,
                        resource_id = resourceId,
                        pay_before = payBefore,
                        seller_signature = sellerSignature,
                        seller_sign_type = "RSA2",
                        seller_unique_id = SellerId
                    },
                    method = new
                    {
                        seller_name = "测试商户",
                        seller_id = SellerId,
                        seller_app_id = AlipayConfig.AppId,
                        goods_name = goodsName,
                        seller_unique_id_key = "seller_id",
                        service_id = ServiceId
                    }
                };

                // 5. Base64URL 编码
                var paymentNeededJson = JsonSerializer.Serialize(paymentNeeded, new JsonSerializerOptions
                {
                    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
                });
                var paymentNeededEncoded = Base64UrlEncode(paymentNeededJson);

                // 6. 构造 402 响应
                var responseBody = new
                {
                    code = "Payment-Needed",
                    message = "需要支付",
                    out_trade_no = outTradeNo,
                    amount = amount,
                    currency = currency,
                    goods_name = goodsName
                };

                Console.WriteLine($"创建支付订单成功：outTradeNo={outTradeNo}, amount={amount}");

                response.StatusCode = 402;
                response.Headers["Payment-Needed"] = paymentNeededEncoded;
                response.ContentType = "application/json; charset=utf-8";
                await response.WriteAsJsonAsync(responseBody);

            }
            catch (Exception ex)
            {
                Console.WriteLine($"创建订单失败：{ex.Message}");
                response.StatusCode = 500;
                response.ContentType = "application/json; charset=utf-8";
                await response.WriteAsJsonAsync(new
                {
                    code = "CREATE_ORDER_ERROR",
                    message = $"创建订单失败：{ex.Message}"
                });
            }
        }

        /// <summary>
        /// 验证支付凭证并交付资源
        /// </summary>
        private static async Task VerifyPaymentAndDeliverResource(HttpResponse response, string paymentProof)
        {
            try
            {
                // 1. 从 Payment-Proof 中解析订单信息
                string paymentProofValue = null;
                string tradeNo = null;
                string clientSession = null;

                try
                {
                    var decodedProof = Base64UrlDecode(paymentProof);
                    using (var proofJson = JsonDocument.Parse(decodedProof))
                    {
                        var root = proofJson.RootElement;
                        
                        if (root.TryGetProperty("protocol", out var protocol))
                        {
                            paymentProofValue = protocol.GetProperty("payment_proof").GetString();
                            tradeNo = protocol.GetProperty("trade_no").GetString();
                        }

                        if (root.TryGetProperty("method", out var method))
                        {
                            clientSession = method.GetProperty("client_session").GetString();
                        }
                    }

                    // 校验必要字段
                    if (string.IsNullOrWhiteSpace(paymentProofValue))
                    {
                        response.StatusCode = 400;
                        response.ContentType = "application/json; charset=utf-8";
                        await response.WriteAsJsonAsync(new
                        {
                            code = "INVALID_PAYMENT_PROOF_FORMAT",
                            message = "Payment-Proof 格式错误：缺少 payment_proof"
                        });
                        return;
                    }

                    if (string.IsNullOrWhiteSpace(tradeNo))
                    {
                        response.StatusCode = 400;
                        response.ContentType = "application/json; charset=utf-8";
                        await response.WriteAsJsonAsync(new
                        {
                            code = "INVALID_PAYMENT_PROOF_FORMAT",
                            message = "Payment-Proof 格式错误：缺少 trade_no"
                        });
                        return;
                    }
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"Payment-Proof 解析失败：{ex.Message}");
                    response.StatusCode = 400;
                    response.ContentType = "application/json; charset=utf-8";
                    await response.WriteAsJsonAsync(new
                    {
                        code = "INVALID_PAYMENT_PROOF_FORMAT",
                        message = $"Payment-Proof 格式错误：{ex.Message}"
                    });
                    return;
                }

                // 2. 调用支付宝 API 验证支付凭证
                var request = new AlipayAipayAgentPaymentVerifyRequest();
                request.BizContent = JsonSerializer.Serialize(new
                {
                    payment_proof = paymentProofValue,
                    trade_no = tradeNo,
                    client_session = clientSession
                }, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });

                var verifyResponse = await AlipayClient.ExecuteAsync(request);

                // 3. 验证失败，返回错误
                if (verifyResponse.Code != "10000")
                {
                    Console.WriteLine($"支付凭证验证失败：{verifyResponse.SubMsg}");
                    response.StatusCode = 400;
                    response.ContentType = "application/json; charset=utf-8";
                    await response.WriteAsJsonAsync(new
                    {
                        code = verifyResponse.SubCode ?? "VERIFY_FAILED",
                        message = verifyResponse.SubMsg ?? "支付凭证验证失败"
                    });
                    return;
                }

                // 4. 验证成功，获取订单信息
                var verifyTradeNo = verifyResponse.TradeNo;
                var verifyOutTradeNo = verifyResponse.OutTradeNo;
                var resourceId = verifyResponse.ResourceId;
                // 当前 demo 在沙箱字段为空时回退到固定资源；生产实现必须从本地订单读取并校验资源 ID。
                var resourceIdVerified = string.IsNullOrWhiteSpace(resourceId) ? ResourcePath : resourceId;
                var active = verifyResponse.Active;

                Console.WriteLine($"支付凭证验证成功：tradeNo={verifyTradeNo}, outTradeNo={verifyOutTradeNo}");

                // 5. 校验凭证有效性（active=true 表示凭证有效）
                if (active != true)
                {
                    Console.WriteLine($"支付凭证无效或已过期：outTradeNo={verifyOutTradeNo}");
                    response.StatusCode = 400;
                    response.ContentType = "application/json; charset=utf-8";
                    await response.WriteAsJsonAsync(new
                    {
                        code = "INVALID_PAYMENT_PROOF",
                        message = "支付凭证无效或已过期"
                    });
                    return;
                }

                // 6. 【TODO】查询订单是否存在（以数据库为准）
                // 7. 【TODO】资源防串校验
                // 8. 【TODO】履约防重放校验

                // 9. 生成资源内容
                var serviceResult = GenerateServiceResource(resourceIdVerified);

                // 10. 【TODO】履约记录落库
                // 11. 【TODO】保存待确认履约状态
                // 实际生产中建议先保存 serviceResult 和 PENDING_CONFIRM 状态。
                // fulfillment.confirm 成功后再标记 FULFILLED；如果确认失败，
                // 允许同一笔 Payment-Proof 重试确认，避免误返回成功。

                var fulfillmentTradeNo = string.IsNullOrWhiteSpace(verifyTradeNo) ? tradeNo : verifyTradeNo;
                Console.WriteLine($"资源已生成，准备发送履约确认：outTradeNo={verifyOutTradeNo}, tradeNo={fulfillmentTradeNo}");

                // 12. 发送履约确认到支付宝，确认成功后才返回成功交付
                var fulfillmentConfirmed = await SendFulfillmentConfirm(fulfillmentTradeNo);
                if (!fulfillmentConfirmed)
                {
                    response.StatusCode = 502;
                    response.ContentType = "application/json; charset=utf-8";
                    await response.WriteAsJsonAsync(new
                    {
                        code = "FULFILLMENT_CONFIRM_FAILED",
                        message = "资源已生成但履约确认失败，请稍后使用同一 Payment-Proof 重试"
                    });
                    return;
                }

                // 12.1 【TODO】履约确认成功后，再标记订单已履约
                // order.FulfillStatus = "FULFILLED";
                // order.FulfilledAt = DateTime.Now;

                Console.WriteLine($"履约确认成功：outTradeNo={verifyOutTradeNo}, tradeNo={fulfillmentTradeNo}");

                // 13. 构造 Payment-Validation Header
                var paymentValidation = new
                {
                    trade_no = fulfillmentTradeNo,
                    out_trade_no = verifyOutTradeNo,
                    validated = true,
                    resource_id = resourceIdVerified
                };

                var paymentValidationJson = JsonSerializer.Serialize(paymentValidation, new JsonSerializerOptions
                {
                    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
                });
                var paymentValidationEncoded = Base64UrlEncode(paymentValidationJson);

                // 14. 返回资源内容
                var responseBody = new
                {
                    resource_id = resourceIdVerified,
                    content = serviceResult,
                    trade_no = fulfillmentTradeNo,
                    out_trade_no = verifyOutTradeNo,
                    already_fulfilled = false,
                    fulfillment_confirmed = true
                };

                response.Headers["Payment-Validation"] = paymentValidationEncoded;
                response.ContentType = "application/json; charset=utf-8";
                await response.WriteAsJsonAsync(responseBody);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"支付凭证验证异常：{ex.Message}");
                response.StatusCode = 500;
                response.ContentType = "application/json; charset=utf-8";
                await response.WriteAsJsonAsync(new
                {
                    code = "VERIFY_FAILED",
                    message = $"支付凭证验证失败：{ex.Message}"
                });
            }
        }

        /// <summary>
        /// 生成服务资源内容
        /// </summary>
        private static object GenerateServiceResource(string resourceId)
        {
            return new
            {
                status = "success",
                service_type = "AI_CONTENT_GENERATION",
                resource_id = resourceId,
                content = "这是 AI 生成的内容示例，可根据实际业务替换为任意数字服务内容",
                generated_at = DateTime.Now.ToString("o")
            };
        }

        /// <summary>
        /// 发送履约确认
        /// </summary>
        private static async Task<bool> SendFulfillmentConfirm(string tradeNo)
        {
            if (string.IsNullOrWhiteSpace(tradeNo))
            {
                Console.WriteLine("履约确认失败：tradeNo 为空");
                return false;
            }

            try
            {
                Console.WriteLine($"开始发送履约确认：tradeNo={tradeNo}");

                var request = new AlipayAipayAgentFulfillmentConfirmRequest();
                request.BizContent = JsonSerializer.Serialize(new
                {
                    trade_no = tradeNo
                }, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });

                var response = await AlipayClient.ExecuteAsync(request);

                if (response.Code == "10000")
                {
                    Console.WriteLine($"履约确认成功：tradeNo={tradeNo}");
                    return true;
                }
                else
                {
                    Console.WriteLine($"履约确认失败：tradeNo={tradeNo}, errorCode={response.SubCode}, errorMsg={response.SubMsg}");
                    return false;
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"履约确认异常：tradeNo={tradeNo}, error={ex.Message}");
                return false;
            }
        }
    }
}
