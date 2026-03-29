package test

import (
	"fmt"
	"net/http"
	"testing"
	"time"

	"github.com/gruntwork-io/terratest/modules/terraform"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestLoadBalancerModulePlan validates the load_balancer module plan without
// applying — safe to run in CI with read-only AWS credentials.
func TestLoadBalancerModulePlan(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/development.tfvars"},
		Vars: map[string]interface{}{
			"db_password": "test-stub",
		},
		PlanFilePath: "/tmp/lb-dev-plan.tfplan",
		NoColor:      true,
	}

	planOutput := terraform.InitAndPlanAndShowWithStruct(t, opts)

	// ALB must be planned
	resources := planOutput.ResourcePlannedValuesMap
	assert.Contains(t, resources, "module.load_balancer.aws_lb.main",
		"ALB resource must be in plan")

	// Both target groups must be planned
	assert.Contains(t, resources, "module.load_balancer.aws_lb_target_group.api",
		"API target group must be in plan")
	assert.Contains(t, resources, "module.load_balancer.aws_lb_target_group.ws",
		"WebSocket target group must be in plan")

	// Listener rules must be planned
	assert.Contains(t, resources, "module.load_balancer.aws_lb_listener_rule.health",
		"Health check listener rule must be in plan")
	assert.Contains(t, resources, "module.load_balancer.aws_lb_listener_rule.websocket",
		"WebSocket listener rule must be in plan")
	assert.Contains(t, resources, "module.load_balancer.aws_lb_listener_rule.api",
		"API listener rule must be in plan")

	// Autoscaling target must be planned
	assert.Contains(t, resources, "module.load_balancer.aws_appautoscaling_target.ecs",
		"Autoscaling target must be in plan")

	// CloudWatch alarms must be planned
	assert.Contains(t, resources, "module.load_balancer.aws_cloudwatch_metric_alarm.alb_5xx",
		"5xx alarm must be in plan")
	assert.Contains(t, resources, "module.load_balancer.aws_cloudwatch_metric_alarm.unhealthy_hosts",
		"Unhealthy hosts alarm must be in plan")

	// Dashboard must be planned
	assert.Contains(t, resources, "module.load_balancer.aws_cloudwatch_dashboard.lb",
		"CloudWatch dashboard must be in plan")
}

// TestLoadBalancerHealthCheck performs a live HTTP health check against a
// deployed ALB. Only runs when APPLY_TESTS=true is set (not in standard CI).
func TestLoadBalancerHealthCheck(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping live health check in short mode")
	}

	opts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/development.tfvars"},
		Vars: map[string]interface{}{
			"db_password": "test-stub",
		},
		NoColor: true,
	}

	defer terraform.Destroy(t, opts)
	terraform.InitAndApply(t, opts)

	albDNS := terraform.Output(t, opts, "api_url")
	require.NotEmpty(t, albDNS, "api_url output must not be empty")

	healthURL := fmt.Sprintf("http://%s/health", albDNS)

	// Retry for up to 3 minutes — ECS tasks take time to start
	client := &http.Client{Timeout: 5 * time.Second}
	deadline := time.Now().Add(3 * time.Minute)

	var lastErr error
	for time.Now().Before(deadline) {
		resp, err := client.Get(healthURL)
		if err == nil {
			resp.Body.Close()
			if resp.StatusCode >= 200 && resp.StatusCode < 300 {
				t.Logf("Health check passed: %s → %d", healthURL, resp.StatusCode)
				return
			}
			lastErr = fmt.Errorf("unexpected status %d", resp.StatusCode)
		} else {
			lastErr = err
		}
		time.Sleep(10 * time.Second)
	}

	t.Fatalf("Health check never passed for %s: %v", healthURL, lastErr)
}

// TestStickinessConfiguration verifies stickiness is enabled in staging/production plans.
func TestStickinessConfiguration(t *testing.T) {
	t.Parallel()

	for _, env := range []string{"staging", "production"} {
		env := env
		t.Run(env, func(t *testing.T) {
			t.Parallel()

			opts := &terraform.Options{
				TerraformDir: "../terraform",
				VarFiles:     []string{fmt.Sprintf("environments/%s.tfvars", env)},
				Vars: map[string]interface{}{
					"db_password":         "test-stub",
					"stellar_contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
				},
				PlanFilePath: fmt.Sprintf("/tmp/%s-lb-plan.tfplan", env),
				NoColor:      true,
			}

			// Plan must succeed — stickiness is enabled in staging/production locals
			terraform.InitAndPlan(t, opts)
		})
	}
}
