#!/usr/bin/env python3
"""
AWS Architecture Diagram Generator for Dispensary Backend

Generates professional AWS architecture diagrams programmatically using the 'diagrams' library.
This approach is better than manual diagram tools because:
- Diagrams are version-controlled alongside your infrastructure code
- Easy to regenerate when architecture changes
- Consistent styling and layout
- Integrates with CI/CD pipelines

Prerequisites:
    1. Python 3.9+
    2. Graphviz (brew install graphviz)
    3. diagrams library (python3 -m pip install diagrams)

Installation:
    brew install graphviz
    python3 -m pip install diagrams

Usage:
    python3 generate_diagram.py

Output:
    Three PNG diagrams are generated in the project root:
    - aws_architecture.png           (Main CQRS/ES architecture)
    - aws_architecture_detailed.png  (Detailed with IAM and DLQs)
    - aws_event_flow.png             (Event flow sequence)

Architecture Overview:
    This project implements Event Sourcing with CQRS pattern on AWS:

    Write Side (Commands):
    - API Gateway V2 → API Lambda (30s) → DynamoDB Event Log

    Event Publishing:
    - DynamoDB Streams → Publisher Lambda (60s) → Kinesis Stream

    Read Side (Queries):
    - Kinesis → Projector Views (60s) → Read Models (DynamoDB Views)
    - Kinesis + S3 Events → Projector Analyzer (300s) → AI Processing

    Key AWS Services:
    - 1x API Gateway V2 (HTTP API, conditional deployment)
    - 4x Lambda Functions (API, Publisher, Projector Views, Projector Analyzer)
    - 3x DynamoDB Tables (Event Log with Streams, Snapshots, Dispenses View)
    - 1x Kinesis Data Stream (PROVISIONED/ON_DEMAND based on environment)
    - 1x S3 Bucket (Encrypted, versioned, multi-format support)
    - 3x SQS Queues (Dead letter queues for error handling)

    Lambda Timeouts:
    - API Lambda: 30s
    - Publisher: 60s
    - Projector Views: 60s
    - Projector Analyzer: 300s (5 minutes for AI processing)
"""

from diagrams import Cluster, Diagram, Edge
from diagrams.aws.analytics import Kinesis
from diagrams.aws.compute import Lambda
from diagrams.aws.database import Dynamodb
from diagrams.aws.general import User
from diagrams.aws.integration import SQS
from diagrams.aws.management import Cloudwatch
from diagrams.aws.network import APIGateway
from diagrams.aws.security import IAM
from diagrams.aws.storage import S3

# Graph attributes for better styling
graph_attr = {
    "fontsize": "14",
    "bgcolor": "white",
    "pad": "0.5",
    "splines": "ortho",
}

node_attr = {
    "fontsize": "12",
}

edge_attr = {
    "fontsize": "10",
}

# Main Architecture Diagram
with Diagram(
    "Dispensary Backend - AWS Event-Sourced Architecture (CQRS/ES)",
    filename="aws_architecture",
    direction="LR",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
    show=False,
):
    user = User("Client/User")

    with Cluster("API Layer"):
        api_gateway = APIGateway("API Gateway V2\n(HTTP API)\n[conditional]")
        api_lambda = Lambda("API Lambda\n(30s timeout)")

    with Cluster("Event Store (Write Model)"):
        event_log = Dynamodb("Event Log\n(with Streams)")
        event_snapshots = Dynamodb("Event Snapshots")

    with Cluster("Event Publishing"):
        publisher_lambda = Lambda("Publisher Lambda\n(60s timeout)")
        kinesis = Kinesis("Kinesis Stream\n(Event Bus)\nPROVISIONED/ON_DEMAND")
        publisher_dlq = SQS("Publisher DLQ")

    with Cluster("Read Model Projectors"):
        projector_views = Lambda("Projector Views\n(60s timeout)")
        projector_analyzer = Lambda("Projector Analyzer\n(300s timeout)")
        views_dlq = SQS("Views DLQ")
        analyzer_dlq = SQS("Analyzer DLQ")

    with Cluster("Read Model & Storage"):
        dispenses_view = Dynamodb("Dispenses View\n(Materialized)")
        prescriptions_s3 = S3(
            "Prescriptions Bucket\n(Encrypted+Versioned)\n.jpg/.jpeg/.png/.pdf"
        )

    with Cluster("Observability"):
        cloudwatch = Cloudwatch("CloudWatch Logs\n(Lambda Execution)")

    # Main flow
    user >> Edge(label="HTTP Request") >> api_gateway
    api_gateway >> Edge(label="Invoke") >> api_lambda
    api_lambda >> Edge(label="Write Events") >> event_log
    api_lambda >> Edge(label="Store State") >> event_snapshots

    # Publishing flow
    event_log >> Edge(label="DynamoDB Streams") >> publisher_lambda
    publisher_lambda >> Edge(label="Publish Events") >> kinesis
    publisher_lambda >> Edge(label="Failures") >> publisher_dlq

    # Projection flows
    kinesis >> Edge(label="Subscribe\n(batch=10)") >> projector_views
    kinesis >> Edge(label="Subscribe\n(batch=1)") >> projector_analyzer

    projector_views >> Edge(label="Update View") >> dispenses_view
    projector_views >> Edge(label="Failures") >> views_dlq

    projector_analyzer >> Edge(label="Write Analysis\nEvents") >> event_log
    projector_analyzer >> Edge(label="Failures") >> analyzer_dlq

    # S3 interactions
    api_lambda >> Edge(label="Upload\n(prescriptions/*)") >> prescriptions_s3
    (
        prescriptions_s3
        >> Edge(label="S3:ObjectCreated:*\n(prescriptions/*.{jpg,jpeg,png,pdf})")
        >> projector_analyzer
    )
    projector_analyzer >> Edge(label="Read File") >> prescriptions_s3

    # Read flow
    api_lambda >> Edge(label="Read Query", style="dashed") >> dispenses_view

    # Logging
    api_lambda >> Edge(label="Logs", style="dotted") >> cloudwatch
    publisher_lambda >> Edge(label="Logs", style="dotted") >> cloudwatch
    projector_views >> Edge(label="Logs", style="dotted") >> cloudwatch
    projector_analyzer >> Edge(label="Logs", style="dotted") >> cloudwatch

print("✅ Generated: aws_architecture.png")

# Detailed Architecture with IAM and Event Flow
with Diagram(
    "Dispensary Backend - Detailed Event Flow",
    filename="aws_architecture_detailed",
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
    show=False,
):
    user = User("Client")

    with Cluster("Security & IAM"):
        iam_role = IAM("Lambda Exec Role\n(DynamoDB+Kinesis\n+S3+SQS)")

    with Cluster("Command Side (Write)"):
        api_gateway = APIGateway("API Gateway V2\n[conditional]")
        api_lambda = Lambda("API Lambda\n(30s)")

        with Cluster("Event Store"):
            event_log = Dynamodb("Event Log\n(Streams Enabled)")
            event_snapshots = Dynamodb("Event Snapshots")

    with Cluster("Event Distribution"):
        publisher_lambda = Lambda("Publisher\n(60s)")
        kinesis = Kinesis("Event Stream\nPROVISIONED/\nON_DEMAND")

        with Cluster("DLQs"):
            pub_dlq = SQS("Publisher DLQ")

    with Cluster("Query Side (Read)"):
        with Cluster("Projectors"):
            proj_views = Lambda("Views Projector\n(60s)")
            proj_analyzer = Lambda("Analyzer Projector\n(300s)")
            views_dlq = SQS("Views DLQ")
            analyzer_dlq = SQS("Analyzer DLQ")

        with Cluster("Read Models"):
            dispenses_view = Dynamodb("Dispenses View")
            prescriptions = S3("Prescriptions\n(Versioned+Encrypted)")

    with Cluster("Observability"):
        cloudwatch = Cloudwatch("CloudWatch Logs")

    # Flows
    user >> api_gateway >> api_lambda
    api_lambda >> Edge(label="AssumeRole", style="dashed") >> iam_role
    api_lambda >> event_log
    api_lambda >> event_snapshots
    api_lambda >> Edge(label="Upload") >> prescriptions

    event_log >> Edge(label="Stream\n(NEW_IMAGE)") >> publisher_lambda >> kinesis
    publisher_lambda >> pub_dlq

    kinesis >> Edge(label="batch=10") >> proj_views >> dispenses_view
    (
        kinesis
        >> Edge(label="batch=1")
        >> proj_analyzer
        >> Edge(label="Append Event")
        >> event_log
    )

    proj_views >> Edge(label="On Failure") >> views_dlq
    proj_analyzer >> Edge(label="On Failure") >> analyzer_dlq

    (
        prescriptions
        >> Edge(
            label="S3:ObjectCreated\nprefix: prescriptions/\nsuffix: .jpg/.jpeg/.png/.pdf"
        )
        >> proj_analyzer
    )
    proj_analyzer >> Edge(label="Read") >> prescriptions

    api_lambda >> Edge(label="Query", style="dashed") >> dispenses_view

    # All lambdas log to CloudWatch
    api_lambda >> Edge(label="Logs", style="dotted") >> cloudwatch
    publisher_lambda >> Edge(label="Logs", style="dotted") >> cloudwatch
    proj_views >> Edge(label="Logs", style="dotted") >> cloudwatch
    proj_analyzer >> Edge(label="Logs", style="dotted") >> cloudwatch

print("✅ Generated: aws_architecture_detailed.png")

# Event Flow Sequence
with Diagram(
    "Dispensary Backend - Event Flow Sequence",
    filename="aws_event_flow",
    direction="LR",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
    show=False,
):
    with Cluster("1. Command"):
        user = User("User")
        api = APIGateway("API GW\n[conditional]")
        cmd_lambda = Lambda("API Lambda\n(30s)")

    with Cluster("2. Event Store"):
        event_log = Dynamodb("Event Log\n(Streams)")

    with Cluster("3. Publishing"):
        publisher = Lambda("Publisher\n(60s)")
        stream = Kinesis("Stream")

    with Cluster("4. Projections"):
        projector1 = Lambda("Views\n(60s)")
        projector2 = Lambda("Analyzer\n(300s)")

    with Cluster("5. Read Models"):
        view_table = Dynamodb("View")
        s3 = S3("S3\n(Multi-format)")

    user >> Edge(label="1. POST /dispenses") >> api
    api >> Edge(label="2. Invoke") >> cmd_lambda
    cmd_lambda >> Edge(label="3. Dispense:Started") >> event_log
    event_log >> Edge(label="4. Stream") >> publisher
    publisher >> Edge(label="5. Publish") >> stream
    stream >> Edge(label="6a. Consume") >> projector1
    stream >> Edge(label="6b. Consume") >> projector2
    projector1 >> Edge(label="7a. Update") >> view_table
    (
        projector2
        >> Edge(label="7b. Process & Write\nDispense:PrescriptionAnalyzed")
        >> event_log
    )
    cmd_lambda >> Edge(label="8. Upload") >> s3
    s3 >> Edge(label="9. Trigger") >> projector2

print("✅ Generated: aws_event_flow.png")

# Simple User Flow Diagram
with Diagram(
    "Dispensary Backend - User Flow (Create → Upload → Edit → Close)",
    filename="user_flow",
    direction="LR",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
    show=False,
):
    user = User("Pharmacist")

    with Cluster("1. Create Dispense"):
        create_api = APIGateway("POST /dispenses")
        create_lambda = Lambda("API Lambda")
        create_db = Dynamodb("Event Log")

    with Cluster("2. Upload Prescription"):
        upload_api = APIGateway("POST /dispenses/{id}/prescription")
        upload_lambda = Lambda("API Lambda")
        upload_s3 = S3("S3 Upload")
        upload_db = Dynamodb("Event Log")

    with Cluster("3. AI Analysis (Auto)"):
        analyzer = Lambda("Analyzer\n(Triggered by S3)")
        analysis_db = Dynamodb("Event Log")

    with Cluster("4. Edit Dispense"):
        edit_api = APIGateway(
            "POST /dispenses/{id}/patient\nPOST /dispenses/{id}/drugs"
        )
        edit_lambda = Lambda("API Lambda")
        edit_db = Dynamodb("Event Log")

    with Cluster("5. Close Dispense"):
        close_api = APIGateway("POST /dispenses/{id}/complete")
        close_lambda = Lambda("API Lambda")
        close_db = Dynamodb("Event Log")

    # Flow 1: Create
    user >> Edge(label="1. Create\nDispense") >> create_api
    create_api >> create_lambda
    create_lambda >> Edge(label="Dispense:Started") >> create_db

    # Flow 2: Upload
    user >> Edge(label="2. Upload\nPrescription") >> upload_api
    upload_api >> upload_lambda >> upload_s3
    upload_lambda >> Edge(label="Prescription:Uploaded") >> upload_db

    # Flow 3: Auto Analysis
    upload_s3 >> Edge(label="3. S3 Trigger\n(Auto)") >> analyzer
    analyzer >> Edge(label="Prescription:Analyzed") >> analysis_db

    # Flow 4: Edit
    user >> Edge(label="4. Add Patient\n& Drugs") >> edit_api
    edit_api >> edit_lambda
    edit_lambda >> Edge(label="Patient:Added\nDrugs:Added") >> edit_db

    # Flow 5: Close
    user >> Edge(label="5. Complete\nDispense") >> close_api
    close_api >> close_lambda
    close_lambda >> Edge(label="Dispense:Completed") >> close_db

print("✅ Generated: user_flow.png")

print("\n📊 Diagrams generated successfully!")
print("\nFiles created:")
print("  - aws_architecture.png (Main architecture)")
print("  - aws_architecture_detailed.png (Detailed with IAM)")
print("  - aws_event_flow.png (Event flow sequence)")
print("  - user_flow.png (User workflow: Create → Upload → Edit → Close)")
print("\n📝 Infrastructure Details:")
print("  Lambda Timeouts:")
print("    - API Lambda: 30s")
print("    - Publisher: 60s")
print("    - Projector Views: 60s")
print("    - Projector Analyzer: 300s (AI processing)")
print("\n  S3 Configuration:")
print("    - Versioning: Enabled")
print("    - Encryption: AES256")
print("    - Supported formats: .jpg, .jpeg, .png, .pdf")
print("    - Event filter: prescriptions/* prefix")
print("\n  Kinesis Stream:")
print("    - Mode: PROVISIONED (dev/local) / ON_DEMAND (prod)")
print("    - Retention: 24h (dev/local) / 48h (prod)")
print("    - Shards: 1 (dev/local) / auto-scaled (prod)")
print("\n  API Gateway:")
print("    - Type: HTTP API (v2)")
print("    - Deployment: Conditional (disabled for LocalStack free tier)")
print("\n📦 To install dependencies:")
print("  brew install graphviz")
print("  pip install diagrams")
