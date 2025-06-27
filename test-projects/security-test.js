// Test file for security vulnerability detection

// SQL Injection vulnerability
function unsafeQuery(userId) {
    const query = "SELECT * FROM users WHERE id = " + userId;
    return executeSQL(query);
}

// XSS vulnerabilities
function displayUserContent(userInput) {
    document.getElementById('content').innerHTML = userInput + '<div>Welcome!</div>';
    document.write('<script>alert("' + userInput + '")</script>');
    eval('var result = ' + userInput);
}

// CSRF vulnerability - POST form without token
const formHTML = `
<form method="post" action="/transfer">
    <input name="amount" type="number">
    <input name="account" type="text">
    <button type="submit">Transfer</button>
</form>
`;

// AJAX without CSRF protection
$.post('/api/delete', { id: userInput });
fetch('/api/update', { method: 'POST', body: JSON.stringify(data) });

// Hardcoded credentials
const API_KEY = "sk-1234567890abcdef1234567890abcdef";
const password = "admin123";
const secret_key = "super-secret-key";

// Weak crypto
const hash = md5(password);
const randomValue = Math.random();

// Command injection
const userCommand = req.body.command;
exec("ls -la " + userCommand);

// Sensitive data in URL
const loginUrl = `https://api.example.com/login?password=${userPassword}&token=${apiToken}`;

// Debug exposure
console.log("User password:", password, "API key:", API_KEY); 