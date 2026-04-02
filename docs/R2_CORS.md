# R2 CORS for browser uploads

When the admin **Add Product** page uploads images from the browser, the frontend sends a `PUT` request to the presigned R2 URL. The browser treats this as cross-origin (e.g. from `http://localhost:3000` to `*.r2.cloudflarestorage.com`) and requires the bucket to return CORS headers. Without a CORS policy, you get:

```text
Access to fetch at 'https://…r2.cloudflarestorage.com/…' from origin 'http://localhost:3000'
has been blocked by CORS policy: No 'Access-Control-Allow-Origin' header is present on the requested resource.
```

## Fix: add a CORS policy on the R2 bucket

1. In the **Cloudflare dashboard**, go to **R2** → select your bucket (e.g. `sudattas-designer-boutique-product-images`) → **Settings**.
2. Under **CORS Policy**, click **Add CORS policy**.
3. Paste the following JSON (adjust origins if needed):

```json
[
  {
    "AllowedOrigins": [
      "http://localhost:3000",
      "https://your-production-domain.com"
    ],
    "AllowedMethods": ["PUT"],
    "AllowedHeaders": ["Content-Type"],
    "ExposeHeaders": ["ETag"],
    "MaxAgeSeconds": 3600
  }
]
```

- **AllowedOrigins**: Add every origin that will open the admin app (e.g. `http://localhost:3000` for dev and your production URL). No trailing slash; no path.
- **AllowedMethods**: `PUT` is required for presigned uploads.
- **AllowedHeaders**: Must include `Content-Type` (the frontend sends it with the file). Do not use `"*"` on R2.
- **ExposeHeaders** / **MaxAgeSeconds**: Optional but recommended.

4. Save. CORS can take up to ~30 seconds to apply.

After this, browser uploads from the Add Product page should succeed.
