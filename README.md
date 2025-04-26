## Setup & Building
```bash
cargo install cargo-watch
cd app-service
cargo build
cd ..
cd auth-service
cargo build
cd ..
```

## Run servers locally (Manually)
#### App service
```bash
cd app-service
cargo watch -q -c -w src/ -w assets/ -w templates/ -x run
```

visit http://localhost:8000

#### Auth service
```bash
cd auth-service
cargo watch -q -c -w src/ -w assets/ -x run
```

visit http://localhost:3000

## Run servers locally (Docker)
```bash
./docker.bat
```

During local development, visit http://localhost:8000 and http://localhost:3000

## Production Deployment with Nginx and SSL

The production setup uses Nginx as a reverse proxy with SSL certificates managed by Certbot, all running in Docker containers.

### How It Works

1. The GitHub Actions workflow automatically:
   - Builds and tests the services
   - Pushes Docker images to Docker Hub
   - Creates necessary directories on the droplet
   - Deploys the compose file and Nginx configuration
   - Sets up dummy certificates if needed
   - Obtains real SSL certificates from Let's Encrypt
   - Starts all services in Docker containers

2. The containerized setup includes:
   - App service (exposed via Nginx at /app/)
   - Auth service (exposed via Nginx at /auth/)
   - Nginx (handling SSL and routing)
   - Certbot (automatic certificate renewal)

### SSL Certificate Management

SSL certificates are automatically renewed using a system cron job that runs twice daily (at 12 AM and 12 PM). The cron job:
1. Runs Certbot in a temporary container to check for and process renewals
2. Reloads the Nginx container to pick up any renewed certificates

This approach is more reliable than running Certbot as a long-lived container and eliminates deployment hangs.

### Manual Certificate Operations

If you need to manually manage certificates on the droplet:

1. To force certificate renewal:
```bash
docker compose run --rm --entrypoint "certbot renew --force-renewal" certbot
docker compose exec nginx nginx -s reload
```

2. To add a new domain:
```bash
docker compose run --rm --entrypoint "certbot certonly --webroot --webroot-path /var/www/certbot \
  --email admin@biosek.cz -d new-domain.example.com \
  --agree-tos --no-eff-email" certbot
```

3. After adding a new domain, update the Nginx configuration in `nginx/conf.d/app.conf` and reload:
```bash
docker compose exec nginx nginx -s reload
```

### Checking Certificate Status

To check the status of your SSL certificates:
```bash
docker compose run --rm --entrypoint "certbot certificates" certbot
```
