// Login request
fetch('http://localhost:3000/login', {
    method: 'POST'
  }).then(response => {
    if (response.ok) {
      // Extract JWT token from response
      return response.text();
    } else {
      throw new Error('Failed to login');
    }
  }).then(token => {
    // Store token in local storage or session storage
    localStorage.setItem('jwt_token', token);
  }).catch(error => {
    console.error('Error:', error);
  });
  
  // Verify request
  const token = localStorage.getItem('jwt_token');
  if (token) {
    fetch('http://localhost:3000/verify', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ token }),
    }).then(response => {
      if (response.ok) {
        // Handle successful token verification
        return response.text();
      } else {
        throw new Error('Invalid token');
      }
    }).then(data => {
      console.log('Token verified:', data);
    }).catch(error => {
      console.error('Error:', error);
    });
  }
  