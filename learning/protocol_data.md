http://localhost:9222/json

```
{
   "description": "",
   "devtoolsFrontendUrl": "/devtools/inspector.html?ws=localhost:9222/devtools/page/F6E874EA9CEDB6682EE90EF6877B7318",
   "faviconUrl": "https://cdn.sstatic.net/Sites/stackoverflow/img/favicon.ico?v=4f32ecc8f43d",
   "id": "F6E874EA9CEDB6682EE90EF6877B7318",
   "title": "command line - How to create a Chrome profile programmatically? - Stack Overflow",
   "type": "page",
   "url": "https://stackoverflow.com/questions/31067404/how-to-create-a-chrome-profile-programmatically",
   "webSocketDebuggerUrl": "ws://localhost:9222/devtools/page/F6E874EA9CEDB6682EE90EF6877B7318"
}
```

git pull https://github.com/atroche/rust-headless-chrome.git

## About invoke methods.

When invoke a method on target the result comes later, But the result contains no information about respone except a matching call id. So we can guess the response or save the information on call id before invoke the method.

## About chained method call.

If archive task c need invoke b first, and b need a be called first how can we let things get done?
we should save a link, a_id -> b_id -> c_id, when a is done b will be called.