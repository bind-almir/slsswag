const response = (statusCode, headers, body) => {
	return {
    statusCode: statusCode || 200,
	  headers: headers || { 
      'Content-Type': 'application/json',
      "Access-Control-Allow-Origin" : "*"
	  },
	  body: JSON.stringify(body)  
	};
} 
  
  module.exports = response;