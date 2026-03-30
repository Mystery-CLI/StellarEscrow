# SSL/TLS + Nginx + LetsEncrypt via EC2 userdata (AWS example)
terraform {
  required_providers {
    aws = { source = "hashicorp/aws" version = "~> 5.0" }
  }
}
provider "aws" { region = var.region }

variable "region" { default = "us-east-1" }
variable "domain" { default = "example.com" }
variable "email" { default = "admin@example.com" }
variable "ami_id" { default = "ami-12345678" }
variable "key_name" { default = "my-key" }

resource "aws_security_group" "web" {
  name        = "web-sg"
  description = "Allow HTTP and HTTPS"
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_instance" "web" {
  ami           = var.ami_id
  instance_type = "t3a.micro"
  key_name      = var.key_name
  vpc_security_group_ids = [aws_security_group.web.id]

  user_data = <<-EOF
    #!/bin/bash
    apt-get update -y
    apt-get install -y nginx certbot python3-certbot-nginx
    rm -f /etc/nginx/sites-enabled/default

    cat > /etc/nginx/sites-available/${var.domain}.conf <<NGINX
    server {
      listen 80 default_server;
      server_name ${var.domain} www.${var.domain};
      location / { return 301 https://$host$request_uri; }
    }

    server {
      listen 443 ssl http2;
      server_name ${var.domain} www.${var.domain};
      root /var/www/html;
      index index.html;

      ssl_certificate /etc/letsencrypt/live/${var.domain}/fullchain.pem;
      ssl_certificate_key /etc/letsencrypt/live/${var.domain}/privkey.pem;

      add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
      add_header X-Frame-Options "DENY" always;
      add_header X-Content-Type-Options "nosniff" always;
      add_header Referrer-Policy "same-origin" always;
      add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; object-src 'none'; frame-ancestors 'none'; base-uri 'self';" always;

      location / { try_files $uri $uri/ =404; }
      location = /health { return 200 'OK'; }
    }
    NGINX

    ln -s /etc/nginx/sites-available/${var.domain}.conf /etc/nginx/sites-enabled/
    systemctl restart nginx
    certbot --nginx --non-interactive --agree-tos --email ${var.email} -d ${var.domain} -d www.${var.domain}

    cat > /usr/local/bin/certbot-renew.sh <<'RENEW'
#!/usr/bin/env bash
set -euo pipefail
certbot renew --quiet --deploy-hook 'systemctl reload nginx'
RENEW
    chmod +x /usr/local/bin/certbot-renew.sh
    echo "0 3 * * * root /usr/local/bin/certbot-renew.sh >> /var/log/certbot-renew.log 2>&1" > /etc/cron.d/certbot-renew
  EOF

  tags = { Name = "web-${var.domain}" }
}
