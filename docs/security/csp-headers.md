# Content Security Policy (CSP)

Content Security Policy (CSP) is one of the most critical security layers for the PWA, acting as a powerful defense against Cross-Site Scripting (XSS) and data injection attacks. A strict policy is non-negotiable.

## Recommended Strict Policy

This policy should be served as an HTTP header for the main application page. It follows the principle of least privilege.

```http
Content-Security-Policy:
  default-src 'none';
  script-src 'self' 'wasm-unsafe-eval';
  style-src 'self' 'unsafe-inline'; /* For UI frameworks that require it, can be tightened with hashes */
  connect-src 'self' https://api.construct-messenger.com wss://ws.construct-messenger.com;
  img-src 'self' blob: data:;
  media-src 'self' blob:;
  font-src 'self';
  manifest-src 'self';
  form-action 'none';
  base-uri 'self';
  frame-ancestors 'none';
  object-src 'none';
```

### Directive Breakdown

-   `default-src 'none'`: By default, nothing is allowed. We must explicitly whitelist every resource type.
-   `script-src 'self' 'wasm-unsafe-eval'`: Allows scripts from our own origin (`'self'`). `'wasm-unsafe-eval'` is currently required for WebAssembly to compile and run efficiently in many browsers. This is a trade-off, but the attack surface is much smaller than a general `'unsafe-eval'`.
-   `style-src 'self' 'unsafe-inline'`: Allows stylesheets from our own origin. `'unsafe-inline'` is often needed for modern UI frameworks. This can be made stricter by using hashes or nonces if the framework supports it.
-   `connect-src 'self' ...`: Defines the allowed endpoints for our API (REST/HTTPS) and WebSocket connections. All other connections will be blocked by the browser.
-   `img-src`, `media-src`: Allows images and media from our origin, as well as from `blob:` and `data:` URLs, which are necessary for displaying decrypted attachments or user-generated content securely without making network requests.
-   `form-action 'none'`: Prevents any `<form>` submissions. Our app is a single-page application that uses API calls, not traditional form posts.
-   `frame-ancestors 'none'`: Prevents the application from being embedded in an `<iframe>` on another site, protecting against clickjacking attacks.

## Other Security Headers

In addition to CSP, the following HTTP headers should always be set:

-   **`Strict-Transport-Security`**: `HSTS: max-age=63072000; includeSubDomains; preload`
    -   Enforces HTTPS across the entire site and all subdomains for two years.
-   **`X-Content-Type-Options`**: `nosniff`
    -   Prevents the browser from MIME-sniffing a response away from the declared content-type.
-   **`X-Frame-Options`**: `DENY`
    -   An older but still useful header to prevent clickjacking (redundant with `frame-ancestors 'none'`).
-   **`Referrer-Policy`**: `strict-origin-when-cross-origin`
    -   Limits the amount of information sent in the `Referer` header to other origins.
