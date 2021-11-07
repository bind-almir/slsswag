using Amazon.Lambda.Core;
using Amazon.Lambda.APIGatewayEvents;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;

[assembly:LambdaSerializer(typeof(Amazon.Lambda.Serialization.SystemTextJson.DefaultLambdaJsonSerializer))]
namespace AwsDotnetCsharp
{
    public static class Helpers
    {
		public static APIGatewayProxyResponse Response(int statusCode, object body = null)
		{		
			return new APIGatewayProxyResponse
			{
					StatusCode = statusCode,
					Body = body != null
					? JsonConvert.SerializeObject(body, new JsonSerializerSettings { 
							ContractResolver = new CamelCasePropertyNamesContractResolver(),
							NullValueHandling = NullValueHandling.Ignore
					})
					: null
			};
			
		}
    }
}
