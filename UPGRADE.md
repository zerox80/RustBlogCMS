# Upgrade Guide - Version 1.1.0

## ⚠️ Important Notice

Version 1.1.0 includes significant security improvements that require configuration changes before deployment.

## 🔧 Required Steps

### 1. Generate Secure JWT Secret

The JWT secret must now be at least 43 characters of high-entropy data long (roughly 256 bits when base64 encoded):

```bash
# Linux/macOS
export JWT_SECRET=$(openssl rand -base64 48)

# Windows PowerShell
$env:JWT_SECRET = [Convert]::ToBase64String((1..48 | ForEach-Object { Get-Random -Maximum 256 }))
```

### 2. Set Strong Admin Credentials

Admin password must be at least 12 characters:

```bash
# Linux/macOS
export ADMIN_USERNAME="admin"
export ADMIN_PASSWORD="YourSecurePassword123!"

# Windows PowerShell
$env:ADMIN_USERNAME = "admin"
$env:ADMIN_PASSWORD = "YourSecurePassword123!"
```

### 3. Update Environment Files

#### For Development (.env)

```bash
# Copy example and edit
cp .env.example .env

# Edit .env and set:
JWT_SECRET=<your-generated-secret>
ADMIN_USERNAME=admin
ADMIN_PASSWORD=<your-secure-password>
```

#### For Docker Deployment

```bash
# Copy example and edit
cp .env.example .env

# Edit .env with your values
# Then deploy:
docker-compose up -d
```

### 4. Rebuild and Restart

#### Development
```bash
# Backend
cd backend
cargo build

# Frontend
npm install
npm run dev
```

#### Docker
```bash
# Stop existing containers
docker-compose down

# Rebuild with new changes
docker-compose build --no-cache

# Start with environment variables
docker-compose up -d
```

## 🔍 Verification

### Check Backend Startup

```bash
# Check logs for successful initialization
docker-compose logs backend | grep -E "(JWT secret|admin user)"
```

You should see:
```
JWT secret initialized successfully
Admin user 'admin' already exists with correct password
```

### Test Login

1. Navigate to `/login`
2. Use your configured `ADMIN_USERNAME` and `ADMIN_PASSWORD`
3. Verify successful login and redirect to `/admin`

## ❌ What Will Break

### Insecure Configurations

The following will **no longer work**:

```bash
# ❌ Will fail - JWT_SECRET not set
docker-compose up

# ❌ Will fail - JWT_SECRET too short
export JWT_SECRET="short"
docker-compose up

# ❌ Will fail - Admin password too short
export ADMIN_PASSWORD="1234567"
docker-compose up
```

### Hardcoded Credentials

- Default credentials are **no longer displayed** in login UI
- No insecure fallback values in code
- Application will **fail to start** without proper configuration

## 🆕 New Features

### Security Headers

All responses now include:
- Content-Security-Policy
- Strict-Transport-Security
- X-Content-Type-Options
- X-Frame-Options

### Input Validation

- Username: max 50 chars, alphanumeric + `_-.`
- Password: max 128 chars
- Tutorial titles: max 200 chars
- Tutorial descriptions: max 1000 chars
- Tutorial content: max 100,000 chars
- Topics: max 20 per tutorial

### Error Handling

- React Error Boundary catches and displays errors gracefully
- Better error messages for authentication failures
- Improved logging for debugging

## 📋 Rollback Plan

If you need to rollback:

```bash
# Stop new version
docker-compose down

# Checkout previous version
git checkout v1.0.0

# Restore old .env with old credentials
# Start old version
docker-compose up -d
```

**Note**: Database is compatible, no migration needed for rollback.

## 🐛 Known Issues

### After Upgrade

1. **Existing JWT tokens are invalid**: Users need to re-login
2. **Old credentials won't work**: Use new configured credentials
3. **Environment variables are mandatory**: App won't start without them

### Solutions

1. Clear localStorage and re-login
2. Use credentials from your `.env` file
3. Follow setup steps above

## 📞 Support

If you encounter issues:

1. Check logs: `docker-compose logs backend`
2. Verify environment variables: `docker-compose config`
3. Review `SECURITY.md` for detailed guidelines
4. Check `CHANGELOG.md` for all changes

## ✅ Post-Upgrade Checklist

- [ ] JWT_SECRET is set and at least 43 characters of high-entropy data
- [ ] ADMIN_PASSWORD is set and at least 12 characters
- [ ] Backend starts successfully
- [ ] Can login with new credentials
- [ ] Admin dashboard is accessible
- [ ] Can create/edit/delete tutorials
- [ ] Environment variables are documented
- [ ] Backups are in place

## 🔐 Security Recommendations

After upgrading:

1. **Change default admin username** if using "admin"
2. **Use strong passwords** (12+ characters with mixed case, numbers, symbols)
3. **Enable HTTPS** in production
4. **Rotate JWT_SECRET** periodically
5. **Monitor logs** for failed login attempts
6. **Keep dependencies updated**

## 📚 Further Reading

- See `SECURITY.md` for complete security guidelines
- See `CHANGELOG.md` for detailed list of changes
- See `.env.example` for configuration options
