const http = require('http');
const https = require('https');
const url = require('url');
const { randomUUID } = require('crypto');

const PORT = 8080;

function sendCors(res, status, body, contentType = 'text/plain') {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS, PUT, DELETE');
  res.setHeader('Access-Control-Allow-Headers', '*');
  res.setHeader('Content-Type', contentType);
  res.writeHead(status);
  res.end(body);
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on('data', chunk => chunks.push(chunk));
    req.on('end', () => resolve(Buffer.concat(chunks)));
    req.on('error', reject);
  });
}

function doLogin(req, res) {
  readBody(req).then(body => {
    let payload;
    try {
      payload = JSON.parse(body.toString('utf-8'));
    } catch {
      return sendCors(res, 400, 'Invalid JSON');
    }
    const uid = String(payload.uid || '');
    const token = String(payload.token || '');
    if (!uid || !token) {
      return sendCors(res, 400, 'uid and token required');
    }
    const duid = `client-${randomUUID()}`;
    const requestId = randomUUID();
    const cookie = [
      `model_device_id=${duid}`,
      'model_os_version=Windows%2010',
      'model_platform_type=2',
      'model_lang=0',
      'timeZone=7200',
      `model_token=${token}`,
      `model_user_id=${uid}`
    ].join('; ');

    const headers = {
      'Host': 'www.crealitycloud.com',
      'Accept': 'application/json',
      'Accept-Language': 'de-DE,de;q=0.9,en-US;q=0.8,en;q=0.7',
      'Content-Type': 'application/json',
      'Origin': 'https://www.crealitycloud.com',
      'Sec-Fetch-Dest': 'empty',
      'Sec-Fetch-Mode': 'cors',
      'Sec-Fetch-Site': 'same-origin',
      'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36',
      '__cxy_app_ch_': 'Chrome 149.0.0.0',
      '__cxy_app_id_': 'cxy-gen2',
      '__cxy_app_ver_': '7.3.10',
      '__cxy_brand_': 'creality',
      '__cxy_duid_': duid,
      '__cxy_os_lang_': '0',
      '__cxy_os_ver_': 'Windows 10',
      '__cxy_platform_': '2',
      '__cxy_requestid_': requestId,
      '__cxy_timezone_': '7200',
      '__cxy_token_': token,
      '__cxy_uid_': uid,
      'Cookie': cookie,
      'Connection': 'close'
    };

    console.log(`[proxy] POST /login for uid=${uid}`);
    const proxyReq = https.request({
      hostname: 'www.crealitycloud.com',
      path: '/api/rest/im/account/login',
      method: 'POST',
      headers,
      rejectUnauthorized: false
    }, proxyRes => {
      const chunks = [];
      proxyRes.on('data', chunk => chunks.push(chunk));
      proxyRes.on('end', () => {
        const data = Buffer.concat(chunks);
        console.log(`[proxy] response ${proxyRes.statusCode}: ${data.slice(0, 200).toString()}`);
        sendCors(res, proxyRes.statusCode, data, proxyRes.headers['content-type'] || 'application/json');
      });
    });
    proxyReq.on('error', err => {
      console.error('[proxy] error:', err);
      sendCors(res, 500, 'Proxy error: ' + err.message);
    });
    proxyReq.write('{}');
    proxyReq.end();
  });
}

function doProxy(req, res) {
  const target = new URL(req.url, 'http://localhost').searchParams.get('url');
  if (!target) {
    return sendCors(res, 400, 'Missing ?url=...');
  }
  const parsed = url.parse(target);
  const options = {
    hostname: parsed.hostname,
    port: parsed.port || (parsed.protocol === 'https:' ? 443 : 80),
    path: parsed.path,
    method: req.method,
    headers: { ...req.headers, host: parsed.host },
    rejectUnauthorized: false
  };
  delete options.headers.origin;
  delete options.headers.referer;

  const proxyReq = (parsed.protocol === 'https:' ? https : http).request(options, proxyRes => {
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS, PUT, DELETE');
    res.setHeader('Access-Control-Allow-Headers', '*');
    res.writeHead(proxyRes.statusCode, proxyRes.headers);
    proxyRes.pipe(res);
  });

  proxyReq.on('error', err => {
    sendCors(res, 500, 'Proxy error: ' + err.message);
  });

  req.pipe(proxyReq);
}

const server = http.createServer((req, res) => {
  if (req.method === 'OPTIONS') {
    return sendCors(res, 200, '');
  }
  if (req.method === 'POST' && req.url === '/login') {
    return doLogin(req, res);
  }
  if (req.url.startsWith('/?')) {
    return doProxy(req, res);
  }
  sendCors(res, 400, 'Use POST /login or ?url=...');
});

server.listen(PORT, () => {
  console.log(`CORS proxy running at http://localhost:${PORT}/`);
  console.log(`In the debug tool use: http://localhost:${PORT}/`);
});
