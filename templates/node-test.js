const { handler } = require('../[function-name]');

test('function handler', async () => {

  try{
    const result = await handler();
    const { statusCode, body } = result;
    const { message } = JSON.parse(body);
    expect(statusCode).toBe(501);
    expect(message).toBe('not implemented!');
  } catch(err) {
    expect(e).toMatch('error');
  }

});