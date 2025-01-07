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

Output of the tool is evaluated and relevant environment variables are set. In case of this example the output can be:

    export TF_VAR_vpc_cidr=$'10.200.0.0/24'
    export TF_VAR_subnet_cidrs=$'[ \"10.200.0.0/25\", \"10.200.0.128/25\" ]'

Terraform picks values for its input variables from environment variables with TF_VAR_ prefix. 
You can put all the variables into a Vault secret and not provide a .tfvars file at all, or you can put a subset of sensitive variables into a secret and set the rest in a .tfvars file.
### Installation
Install the tool on Debian/Ubuntu from .deb package provided in release:

    wget https://github.com/chushkintech/vttf/releases/download/v0.2.0/vttf_0.2.0_amd64.deb
    sudo dpkg -i vttf_0.2.0_amd64.deb

### Usage

    Usage: vttf [OPTIONS] <MOUNT_POINT> <PATH>
    
    Arguments:
      <MOUNT_POINT>  Secret mount point
      <PATH>         Vault secret path
    
    Options:
          --vault-address <ADDR>   Vault address
          --vault-token <TOKEN>    Vault token
          --value-prefix <PREFIX>  Prefixes for output [default: TF_VAR_]
      -h, --help                   Print help
      -V, --version                Print version

If --vault-address and/or --vault-token are not provided, the tool will look for VAULT_ADDR and VAULT_TOKEN environment variables. Vault address and token environment variables take precedence over command line parameters.

