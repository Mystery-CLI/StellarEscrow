# ALB + autoscaling + load balancing
resource "aws_lb" "app" {
  name               = "app-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = var.subnets
}

resource "aws_lb_target_group" "app" {
  name     = "app-tg"
  port     = 80
  protocol = "HTTP"
  vpc_id   = var.vpc_id

  health_check {
    enabled             = true
    matcher             = "200-399"
    interval            = 15
    path                = "/health"
    timeout             = 5
    healthy_threshold   = 2
    unhealthy_threshold = 3
  }

  stickiness {
    type            = "lb_cookie"
    enabled         = true
    cookie_duration = 86400
  }
}

resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.app.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.app.arn
  }
}

resource "aws_autoscaling_group" "app_asg" {
  name               = "app-asg"
  min_size           = 2
  max_size           = 8
  desired_capacity   = 3
  vpc_zone_identifier = var.subnets
  launch_template {
    id      = aws_launch_template.app.id
    version = "$Latest"
  }
  target_group_arns = [aws_lb_target_group.app.arn]
}

resource "aws_cloudwatch_metric_alarm" "alb_request_count" {
  alarm_name          = "ALB-RequestCountHigh"
  namespace           = "AWS/ApplicationELB"
  metric_name         = "RequestCount"
  dimensions = { LoadBalancer = aws_lb.app.arn_suffix }
  period              = 60
  evaluation_periods  = 5
  statistic           = "Sum"
  threshold           = 5000
  comparison_operator = "GreaterThanThreshold"
}
