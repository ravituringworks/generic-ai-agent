#!/bin/bash
# The Agency - Kubernetes Deployment Script

set -e

# Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

NAMESPACE="the-agency"
IMAGE_TAG="${IMAGE_TAG:-v0.2.0}"
IMAGE_REPO="${IMAGE_REPO:-the-agency}"

echo -e "${GREEN}The Agency - Kubernetes Deployment${NC}"
echo -e "${GREEN}===================================${NC}"
echo ""

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}Error: kubectl not found${NC}"
    echo "Please install kubectl first"
    exit 1
fi

# Check cluster connection
echo -e "${CYAN}Checking cluster connection...${NC}"
if ! kubectl cluster-info &> /dev/null; then
    echo -e "${RED}Error: Cannot connect to Kubernetes cluster${NC}"
    exit 1
fi

CLUSTER=$(kubectl config current-context)
echo -e "  ${GREEN}✓${NC} Connected to cluster: $CLUSTER"

# Build Docker image
echo ""
echo -e "${CYAN}Building Docker image...${NC}"
read -p "Build Docker image? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    docker build -t "${IMAGE_REPO}:${IMAGE_TAG}" .
    echo -e "  ${GREEN}✓${NC} Image built: ${IMAGE_REPO}:${IMAGE_TAG}"
    
    # Optional: Push to registry
    read -p "Push image to registry? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${CYAN}Pushing image...${NC}"
        docker push "${IMAGE_REPO}:${IMAGE_TAG}"
        echo -e "  ${GREEN}✓${NC} Image pushed"
    fi
fi

# Create namespace
echo ""
echo -e "${CYAN}Creating namespace...${NC}"
kubectl apply -f k8s/namespace.yaml
echo -e "  ${GREEN}✓${NC} Namespace created/updated"

# Apply ConfigMap
echo ""
echo -e "${CYAN}Applying ConfigMap...${NC}"
kubectl apply -f k8s/configmap.yaml
echo -e "  ${GREEN}✓${NC} ConfigMap applied"

# Create PVC
echo ""
echo -e "${CYAN}Creating PersistentVolumeClaim...${NC}"
kubectl apply -f k8s/pvc.yaml
echo -e "  ${GREEN}✓${NC} PVC created"

# Deploy application
echo ""
echo -e "${CYAN}Deploying application...${NC}"
kubectl apply -f k8s/deployment.yaml
echo -e "  ${GREEN}✓${NC} Deployment created/updated"

# Create Service
echo ""
echo -e "${CYAN}Creating Service...${NC}"
kubectl apply -f k8s/service.yaml
echo -e "  ${GREEN}✓${NC} Service created/updated"

# Create HPA
echo ""
echo -e "${CYAN}Creating HorizontalPodAutoscaler...${NC}"
kubectl apply -f k8s/hpa.yaml
echo -e "  ${GREEN}✓${NC} HPA created/updated"

# Optional: Create Ingress
echo ""
read -p "Deploy Ingress? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Note: Make sure to update the host in k8s/ingress.yaml${NC}"
    kubectl apply -f k8s/ingress.yaml
    echo -e "  ${GREEN}✓${NC} Ingress created/updated"
fi

# Wait for deployment
echo ""
echo -e "${CYAN}Waiting for deployment to be ready...${NC}"
kubectl rollout status deployment/the-agency -n $NAMESPACE --timeout=5m

# Check status
echo ""
echo -e "${CYAN}Deployment status:${NC}"
kubectl get all -n $NAMESPACE

echo ""
echo -e "${GREEN}Deployment complete!${NC}"
echo ""
echo -e "${CYAN}Useful commands:${NC}"
echo -e "  View pods:        kubectl get pods -n $NAMESPACE"
echo -e "  View logs:        kubectl logs -f deployment/the-agency -n $NAMESPACE"
echo -e "  Port forward:     kubectl port-forward svc/the-agency 8080:8080 -n $NAMESPACE"
echo -e "  Scale replicas:   kubectl scale deployment/the-agency --replicas=3 -n $NAMESPACE"
echo -e "  Delete all:       kubectl delete namespace $NAMESPACE"
echo ""

# Port forward option
read -p "Setup port forwarding to access locally? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo -e "${CYAN}Setting up port forwarding...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""
    kubectl port-forward svc/the-agency 8080:8080 -n $NAMESPACE
fi
