variable "region" {
  type = string
}

variable "fqdn" {
  type = string
}

variable "fqdn_subdomain" {
  type     = string
  nullable = true
  default  = null
}
