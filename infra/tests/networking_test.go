// Infrastructure tests using Terratest.
// Run: go test -v -timeout 30m ./...
package test

import (
	"testing"

	"github.com/gruntwork-io/terratest/modules/terraform"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNetworkingModule(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform/modules/networking",
		Vars: map[string]interface{}{
			"name_prefix":        "test-networking",
			"vpc_cidr":           "10.99.0.0/16",
			"availability_zones": []string{"us-east-1a", "us-east-1b"},
		},
		NoColor: true,
	}

	defer terraform.Destroy(t, opts)
	terraform.InitAndApply(t, opts)

	vpcID := terraform.Output(t, opts, "vpc_id")
	require.NotEmpty(t, vpcID, "vpc_id output must not be empty")

	publicSubnets := terraform.OutputList(t, opts, "public_subnet_ids")
	assert.Len(t, publicSubnets, 2, "expected 2 public subnets")

	privateSubnets := terraform.OutputList(t, opts, "private_subnet_ids")
	assert.Len(t, privateSubnets, 2, "expected 2 private subnets")
}

func TestDevelopmentEnvironmentPlan(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/development.tfvars"},
		Vars: map[string]interface{}{
			"db_password": "test-password-stub",
		},
		PlanFilePath: "/tmp/dev-plan.tfplan",
		NoColor:      true,
	}

	planOutput := terraform.InitAndPlanAndShowWithStruct(t, opts)

	resources := planOutput.ResourcePlannedValuesMap
	assert.Contains(t, resources, "module.networking.aws_vpc.main",
		"VPC must be in plan")
}

func TestStagingEnvironmentPlan(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/staging.tfvars"},
		Vars: map[string]interface{}{
			"db_password":         "test-password-stub",
			"stellar_contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
		},
		PlanFilePath: "/tmp/staging-plan.tfplan",
		NoColor:      true,
	}

	terraform.InitAndPlan(t, opts)
}

func TestProductionEnvironmentPlan(t *testing.T) {
	t.Parallel()

	opts := &terraform.Options{
		TerraformDir: "../terraform",
		VarFiles:     []string{"environments/production.tfvars"},
		Vars: map[string]interface{}{
			"db_password":         "test-password-stub",
			"stellar_contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
		},
		PlanFilePath: "/tmp/prod-plan.tfplan",
		NoColor:      true,
	}

	terraform.InitAndPlan(t, opts)
}
