const response = require('response');

const handler = async (event, context) => {
  return response(501, null, { message: 'not implemented!' });
}

module.exports = { handler }