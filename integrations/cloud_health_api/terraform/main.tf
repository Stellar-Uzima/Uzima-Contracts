/*
  Minimal Terraform example that creates Kubernetes deployment and service via the Kubernetes provider.
  This is an example; it assumes you have a kubeconfig and the kubernetes provider configured.
*/
terraform {
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
  }
}

provider "kubernetes" {
  # Configure via KUBECONFIG or environment (left as example)
}

resource "kubernetes_deployment" "uzima_cloud_health_api" {
  metadata {
    name = "uzima-cloud-health-api"
    labels = { app = "uzima-cloud-health-api" }
  }

  spec {
    replicas = 1
    selector { match_labels = { app = "uzima-cloud-health-api" } }
    template {
      metadata { labels = { app = "uzima-cloud-health-api" } }
      spec {
        container {
          image = "uzima-cloud-health-api:latest"
          name  = "uzima-cloud-health-api"
          port { container_port = 3000 }
          env {
            name  = "UZIMA_USE_GOOGLE_REAL"
            value = "0"
          }
        }
      }
    }
  }
}

resource "kubernetes_service" "uzima_cloud_health_api" {
  metadata { name = "uzima-cloud-health-api" }
  spec {
    selector = { app = "uzima-cloud-health-api" }
    port {
      port        = 3000
      target_port = 3000
    }
  }
}
