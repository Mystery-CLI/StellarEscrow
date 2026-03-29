package test

import (
	"fmt"
	"testing"

	"github.com/gruntwork-io/terratest/modules/terraform"
	"github.com/stretchr/testify/assert"
)

// TestDatabaseModulePlan validates the database module plan for all environments.
func TestDatabaseModulePlan(t *testing.T) {
	t.Parallel()

	for _, env := range []string{"development", "staging", "production"} {
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
				PlanFilePath: fmt.Sprintf("/tmp/%s-db-plan.tfplan", env),
				NoColor:      true,
			}

			planOutput := terraform.InitAndPlanAndShowWithStruct(t, opts)
			resources := planOutput.ResourcePlannedValuesMap

			assert.Contains(t, resources, "module.database.aws_db_instance.primary",
				"Primary RDS instance must be in plan")
			assert.Contains(t, resources, "module.database.aws_db_parameter_group.main",
				"Parameter group must be in plan")
			assert.Contains(t, resources, "module.database.aws_iam_role.rds_enhanced_monitoring",
				"Enhanced monitoring IAM role must be in plan")
			assert.Contains(t, resources, "module.database.aws_cloudwatch_metric_alarm.cpu_high",
				"CPU alarm must be in plan")
			assert.Contains(t, resources, "module.database.aws_cloudwatch_metric_alarm.free_storage_low",
				"Free storage alarm must be in plan")
			assert.Contains(t, resources, "module.database.aws_cloudwatch_metric_alarm.connections_high",
				"Connections alarm must be in plan")
			assert.Contains(t, resources, "module.database.aws_cloudwatch_dashboard.db",
				"Database dashboard must be in plan")
		})
	}
}

// TestDatabaseReplicaInProduction verifies the read replica is only planned for production.
func TestDatabaseReplicaInProduction(t *testing.T) {
	t.Parallel()

	prodOpts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/production.tfvars"},
		Vars: map[string]interface{}{
			"db_password":         "test-stub",
			"stellar_contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
		},
		PlanFilePath: "/tmp/prod-db-replica-plan.tfplan",
		NoColor:      true,
	}

	prodPlan := terraform.InitAndPlanAndShowWithStruct(t, prodOpts)
	resources := prodPlan.ResourcePlannedValuesMap

	assert.Contains(t, resources, "module.database.aws_db_instance.replica[0]",
		"Read replica must be planned for production")
	assert.Contains(t, resources, "module.database.aws_cloudwatch_metric_alarm.replica_lag[0]",
		"Replica lag alarm must be planned for production")
}

// TestDatabaseMultiAZInProduction verifies multi-AZ is enabled in production plan.
func TestDatabaseMultiAZInProduction(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/production.tfvars"},
		Vars: map[string]interface{}{
			"db_password":         "test-stub",
			"stellar_contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
		},
		PlanFilePath: "/tmp/prod-multiaz-plan.tfplan",
		NoColor:      true,
	}

	planOutput := terraform.InitAndPlanAndShowWithStruct(t, opts)
	resources := planOutput.ResourcePlannedValuesMap

	primary, ok := resources["module.database.aws_db_instance.primary"]
	if assert.True(t, ok, "Primary RDS instance must be in plan") {
		assert.Equal(t, true, primary.AttributeValues["multi_az"],
			"multi_az must be true in production")
		assert.Equal(t, float64(14), primary.AttributeValues["backup_retention_period"],
			"backup_retention_period must be 14 days in production")
	}
}
