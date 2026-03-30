# AWS CloudFront CDN
resource "aws_s3_bucket" "static" {
  bucket = "myapp-static-${random_id.bucket_suffix.hex}"
  acl    = "private"
  force_destroy = true
}

resource "aws_cloudfront_origin_access_identity" "oai" {
  comment = "OAI for static bucket"
}

resource "aws_cloudfront_distribution" "cdn" {
  enabled             = true
  is_ipv6_enabled     = true
  price_class         = "PriceClass_All"
  default_root_object = "index.html"

  origin {
    domain_name = aws_s3_bucket.static.bucket_regional_domain_name
    origin_id   = "s3-static-origin"
    s3_origin_config {
      origin_access_identity = aws_cloudfront_origin_access_identity.oai.cloudfront_access_identity_path
    }
  }

  default_cache_behavior {
    target_origin_id       = "s3-static-origin"
    viewer_protocol_policy = "redirect-to-https"
    allowed_methods        = ["GET", "HEAD", "OPTIONS"]
    cached_methods         = ["GET", "HEAD"]
    smooth_streaming       = false

    forwarded_values {
      query_string = false
      cookies { forward = "none" }
    }

    min_ttl     = 0
    default_ttl = 3600
    max_ttl     = 86400
  }

  restrictions {
    geo_restriction { restriction_type = "none" }
  }

  viewer_certificate {
    cloudfront_default_certificate = true
  }
}

resource "aws_cloudwatch_metric_alarm" "cloudfront_hit_ratio" {
  alarm_name          = "cloudfront-hit-rate-low"
  namespace           = "AWS/CloudFront"
  metric_name         = "CacheHitRate"
  statistic           = "Average"
  period              = 300
  evaluation_periods  = 3
  threshold           = 80
  comparison_operator = "LessThanThreshold"
  dimensions = {
    DistributionId = aws_cloudfront_distribution.cdn.id
    Region         = "Global"
  }
  alarm_description = "Alert when CloudFront cache hit rate drops below 80%."
}
