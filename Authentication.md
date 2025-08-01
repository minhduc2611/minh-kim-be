```
sequenceDiagram
  participant User
  participant Frontend
  participant Supabase as AuthService
  participant Backend

  %% Email signup/login flow
  (AuthFlow-email-signup 1) User ->> Frontend: Sign up with email & password
  (AuthFlow-email-signup 2) Frontend ->> Supabase: supabase.auth.signUp(email, password)
  (AuthFlow-email-signup 3) Supabase -->> User: Email confirmation link (optional)
  (AuthFlow-email-signup 4) User ->> Supabase: Confirms email (if enabled)
  (AuthFlow-email-signup 5) Supabase -->> Frontend: JWT tokens (access & refresh)
  (AuthFlow-email-signup 6) Frontend ->> Backend: Bearer access_token in header
  (AuthFlow-email-signup 7) Backend ->> Supabase: auth.getUser(token)
  (AuthFlow-email-signup 8) Supabase -->> Backend: Valid user data
  (AuthFlow-email-signup 9) Backend -->> Frontend: Protected resource

```

```
graph TD
    A[Client/Frontend] --> B[Auth Controller]
    B --> C[AuthService]
    C --> D[Supabase Auth Service]
    C --> E[JWT + Weviate Auth Service]

    subgraph "Auth Endpoints"
        F["POST /auth/signup<br/>SignUpRequest"]
        G["POST /auth/login<br/>LoginRequest"]
        H["GET /auth/verify<br/>Bearer Token"]
        I["POST /auth/refresh<br/>RefreshTokenRequest"]
        J["POST /auth/logout<br/>Bearer Token"]
        K["GET /auth/user/{user_id}<br/>Path Parameter"]
    end

    subgraph "Auth Flow Steps"
        L["(1) User signup/login"]
        M["(2) Frontend → Auth Service"]
        N["(3) Email confirmation (optional)"]
        O["(5) JWT tokens returned"]
        P["(6) Bearer token in header"]
        Q["(7) Backend → Auth Service"]
        R["(8) Valid user data"]
        S["(9) Protected resource"]
    end

    subgraph "Response Format"
        T["AuthResponse{<br/>success: bool<br/>data: Option<T><br/>message: Option<String><br/>error: Option<String><br/>}"]
    end

    A --> F
    A --> G
    A --> H
    A --> I
    A --> J
    A --> K

    F --> L
    G --> L
    H --> P

    style F fill:#e1f5fe
    style G fill:#e1f5fe
    style H fill:#f3e5f5
    style I fill:#fff3e0
    style J fill:#ffebee
    style K fill:#e8f5e8
```
