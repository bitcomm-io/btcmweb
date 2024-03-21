// JWT token
const token = localStorage.getItem('jwt_token');

// JSON-RPC 请求
const jsonRpcRequest = {
  jsonrpc: '2.0',
  method: 'exampleMethod',
  params: { token }, // Include JWT token in params
  id: 1
};

// 发送 JSON-RPC 请求到后端
fetch('http://localhost:3000/json-rpc', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify(jsonRpcRequest),
}).then(response => {
  if (response.ok) {
    // 处理成功响应
    return response.json();
  } else {
    throw new Error('Failed to make JSON-RPC request');
  }
}).then(data => {
  // 处理后端返回的数据
  console.log('Response:', data);
}).catch(error => {
  console.error('Error:', error);
});
