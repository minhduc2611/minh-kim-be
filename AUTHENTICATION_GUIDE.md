# Authentication Implementation Guide

## Overview

All Canvas API endpoints now require authentication. We've implemented a custom `AuthenticatedUser` extractor that automatically validates JWT tokens and provides user information to your handlers.

## How It Works

### 1. **Custom Extractor**: `AuthenticatedUser`

- Automatically extracts and validates the `Authorization: Bearer <token>` header
- Verifies the token with your auth service (Supabase)
- Provides authenticated user information to your endpoint handlers
- Returns proper error responses for missing/invalid tokens

### 2. **Protected Endpoints**

All canvas endpoints now require authentication:

- `GET /canvas` - List canvases (uses authenticated user's ID if no author_id provided)
- `POST /canvas` - Create canvas (automatically sets author_id to authenticated user)
- `GET /canvas/{id}` - Get canvas by ID
- `PUT /canvas/{id}` - Update canvas
- `DELETE /canvas/{id}` - Delete canvas

### 3. **Public Endpoints** (No Authentication Required)

- `POST /auth/signup` - Register new user
- `POST /auth/login` - Login user
- `POST /auth/refresh` - Refresh token
- `GET /auth/verify` - Verify token
- `POST /auth/logout` - Logout user
- `GET /auth/user/{user_id}` - Get user by ID

## Usage Examples

### ✅ **Successful Request** (With Authentication)

```bash
# First, get an access token by signing up or logging in
curl 'http://localhost:8080/auth/login' \
  -H 'Content-Type: application/json' \
  --data-raw '{"email":"user@example.com","password":"password123"}'

# Response:
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "...",
    "user": { "id": "123", "email": "user@example.com", ... },
    "expires_in": 3600
  }
}

# Then use the access_token in protected requests
curl 'http://localhost:8080/canvas' \
  -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...' \
  -H 'Content-Type: application/json'

# Response:
{
  "success": true,
  "data": [...],
  "pagination": { ... }
}
```

### ❌ **Failed Request** (Missing Authentication)

```bash
curl 'http://localhost:8080/canvas'

# Response (401 Unauthorized):
{
  "success": false,
  "data": null,
  "message": "Missing or invalid Authorization header. Please provide: Authorization: Bearer <your_token>",
  "error": "MissingToken"
}
```

### ❌ **Failed Request** (Invalid/Expired Token)

```bash
curl 'http://localhost:8080/canvas' \
  -H 'Authorization: Bearer invalid_token'

# Response (401 Unauthorized):
{
  "success": false,
  "data": null,
  "message": "Invalid token: ...",
  "error": "InvalidToken"
}
```

## Implementation Details

### In Your Controllers

```rust
use crate::middleware::AuthenticatedUser;

#[get("/canvas")]
pub async fn get_canvas_list(
    authenticated_user: AuthenticatedUser,  // 🔐 This requires authentication
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    query: web::Query<ListCanvasQuery>,
) -> Result<impl Responder> {
    // authenticated_user.user contains:
    // - user.id: String
    // - user.email: String
    // - user.name: Option<String>
    // - user.roles: Vec<String>

    let user_id = authenticated_user.user.id;
    // ... rest of your logic
}
```

### Automatic User Assignment

For canvas creation, the `author_id` is automatically set to the authenticated user:

```rust
#[post("/canvas")]
pub async fn create_canvas(
    authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    mut req: web::Json<CreateCanvasRequest>,
) -> Result<impl Responder> {
    // Automatically set the author to the authenticated user
    req.author_id = authenticated_user.user.id.clone();

    // Now only the authenticated user can create canvases in their name
    match service.create_canvas(req.into_inner()).await {
        // ...
    }
}
```

## Error Responses

The authentication middleware returns standardized error responses:

| Scenario                     | HTTP Status | Error Code             | Message                                      |
| ---------------------------- | ----------- | ---------------------- | -------------------------------------------- |
| Missing Authorization header | 401         | `MissingToken`         | "Missing or invalid Authorization header..." |
| Invalid token format         | 401         | `InvalidToken`         | "Invalid token: ..."                         |
| Expired token                | 401         | `TokenExpired`         | "Token has expired. Please refresh..."       |
| Service unavailable          | 401         | `ServiceUnavailable`   | "Authentication service not available"       |
| External service error       | 401         | `ExternalServiceError` | "Authentication service error"               |

## Testing Your Implementation

1. **Start your server**: `cargo run`

2. **Test without authentication** (should fail):

```bash
curl 'http://localhost:8080/canvas'
# Expected: 401 with MissingToken error
```

3. **Get an access token**:

```bash
curl 'http://localhost:8080/auth/signup' \
  -H 'Content-Type: application/json' \
  --data-raw '{"email":"test@example.com","password":"password123","name":"Test User"}'
```

4. **Test with authentication** (should succeed):

```bash
curl 'http://localhost:8080/canvas' \
  -H 'Authorization: Bearer YOUR_ACCESS_TOKEN_HERE'
```

## Security Features

✅ **Automatic token validation** on every protected request  
✅ **Standardized error responses** for different failure scenarios  
✅ **User context injection** - every handler gets authenticated user info  
✅ **Protection against** missing, malformed, expired, or invalid tokens  
✅ **Seamless integration** - just add `AuthenticatedUser` parameter to any handler

Your API is now fully secured! 🔐
