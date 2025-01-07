[![Rust](https://github.com/chushkintech/vttf/actions/workflows/rust.yml/badge.svg)](https://github.com/chushkintech/vttf/actions/workflows/rust.yml)
## Hashicorp Vault To Terraform
This utility is designed to provide contents of a Vault secret as environment variables used by Terraform so that there's no need to modify your Terraform to include the Vault provider and relevant code.
### How It Works
The tool is used to extract key-value pairs from a Vault secret and produce a set of "export" commands that set environment variables that Terraform looks up according to the structure of a secret. Environment variable names are prefixed with TF_VAR_ prefix (this can be changed with a command parameter).

For example, if the following variables are declared:

    variable "vpc_cidr" {
        type = string
    }
    
    variable "subnet_cidrs" {
        type = list(string)
    }

and a secret named "infra" with the mount point "secret" exists in Vault, you can populate the secret with keys "vpc_cidr" and "subnet_cidrs" and respective values that you would usually write to .tfvars file.
Having it set, vttf can be used in an automation pipeline in the following way:

    #!/bin/bash
    eval $(vttf --vault-address <VAULT_ADDRESS_HERE> --vault-token <YOUR_TOKEN_HERE> secret infra)
    terraform plan

Output of the tool is evaluated and relevant environment variables are set. In case of this example the output will be:

    export TF_VAR_vpc_cidr=$'10.200.0.0/24'
    export TF_VAR_subnet_cirds=$'[ \"10.200.0.0/25\", \"10.200.0.128/25\" ]'
