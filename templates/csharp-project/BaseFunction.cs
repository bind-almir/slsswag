using Amazon.Lambda.Core;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;
using Amazon.Lambda.APIGatewayEvents;
using System.Threading.Tasks;

namespace AwsDotnetCsharp
{
    public class Handler
    {
        public async Task<APIGatewayProxyResponse> Base(APIGatewayProxyRequest request, ILambdaContext context)
        {            
            object body = JsonConvert.SerializeObject(new
                {
                    message = "not implemented!"
                });
            return Helpers.Response(501, body);
        }
  }
}
