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
    - API Gateway V2 → API Lambda → DynamoDB Event Log

    Event Publishing:
    - DynamoDB Streams → Publisher Lambda → Kinesis Stream

    Read Side (Queries):
    - Kinesis → Projector Lambdas → Read Models (DynamoDB Views)
    - S3 Events → Analyzer Lambda → AI Processing

    Key AWS Services:
    - 1x API Gateway V2 (HTTP API)
    - 4x Lambda Functions (API, Publisher, 2 Projectors)
    - 3x DynamoDB Tables (Event Log with Streams, Snapshots, Views)
    - 1x Kinesis Data Stream (Event distribution)
    - 1x S3 Bucket (Encrypted prescription storage)
    - 3x SQS Queues (Dead letter queues for error handling)"""

from diagrams import Cluster, Diagram, Edge
from diagrams.aws.analytics import Kinesis
from diagrams.aws.compute import Lambda
from diagrams.aws.database import DynamodbTable
from diagrams.aws.general import User
from diagrams.aws.integration import SQS
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
        api_gateway = APIGateway("API Gateway V2\n(HTTP API)")
        api_lambda = Lambda("API Lambda\n(Command Handler)")

    with Cluster("Event Store (Write Model)"):
        event_log = DynamodbTable("Event Log\n(with Streams)")
        event_snapshots = DynamodbTable("Event Snapshots")

    with Cluster("Event Publishing"):
        publisher_lambda = Lambda("Publisher Lambda")
        kinesis = Kinesis("Kinesis Stream\n(Event Bus)")
        publisher_dlq = SQS("Publisher DLQ")

    with Cluster("Read Model Projectors"):
        projector_views = Lambda("Projector Views\n(Read Model)")
        projector_analyzer = Lambda("Projector Analyzer\n(AI Analysis)")
        views_dlq = SQS("Views DLQ")
        analyzer_dlq = SQS("Analyzer DLQ")

    with Cluster("Read Model & Storage"):
        dispenses_view = DynamodbTable("Dispenses View\n(Materialized)")
        prescriptions_s3 = S3("Prescriptions\nBucket")

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

    projector_views >> Edge(label="Update") >> dispenses_view
    projector_views >> Edge(label="Failures") >> views_dlq

    projector_analyzer >> Edge(label="Analyze & Update") >> event_log
    projector_analyzer >> Edge(label="Failures") >> analyzer_dlq

    # S3 interactions
    api_lambda >> Edge(label="Upload") >> prescriptions_s3
    prescriptions_s3 >> Edge(label="S3 Trigger\n(on upload)") >> projector_analyzer
    projector_analyzer >> Edge(label="Read") >> prescriptions_s3

    # Read flow
    api_lambda >> Edge(label="Read Query", style="dashed") >> dispenses_view

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

    with Cluster("Security"):
        iam_role = IAM("Lambda\nExecution Role")

    with Cluster("Command Side (Write)"):
        api_gateway = APIGateway("API Gateway")
        api_lambda = Lambda("API Lambda")

        with Cluster("Event Store"):
            event_log = DynamodbTable("Event Log")
            event_snapshots = DynamodbTable("Snapshots")

    with Cluster("Event Distribution"):
        publisher_lambda = Lambda("Publisher")
        kinesis = Kinesis("Event Stream")

        with Cluster("DLQs"):
            pub_dlq = SQS("Publisher DLQ")

    with Cluster("Query Side (Read)"):
        with Cluster("Projectors"):
            proj_views = Lambda("Views Projector")
            proj_analyzer = Lambda("Analyzer Projector")
            views_dlq = SQS("Views DLQ")
            analyzer_dlq = SQS("Analyzer DLQ")

        with Cluster("Read Models"):
            dispenses_view = DynamodbTable("Dispenses View")
            prescriptions = S3("Prescriptions")

    # Flows
    user >> api_gateway >> api_lambda
    api_lambda >> iam_role
    api_lambda >> event_log
    api_lambda >> event_snapshots
    api_lambda >> prescriptions

    event_log >> Edge(label="Stream") >> publisher_lambda >> kinesis
    publisher_lambda >> pub_dlq

    kinesis >> proj_views >> dispenses_view
    kinesis >> proj_analyzer >> event_log

    proj_views >> views_dlq
    proj_analyzer >> analyzer_dlq

    prescriptions >> Edge(label="Trigger") >> proj_analyzer
    proj_analyzer >> prescriptions

    api_lambda >> Edge(label="Query", style="dashed") >> dispenses_view

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
        api = APIGateway("API")
        cmd_lambda = Lambda("API Lambda")

    with Cluster("2. Event Store"):
        events = DynamodbTable("Event Log")

    with Cluster("3. Publishing"):
        publisher = Lambda("Publisher")
        stream = Kinesis("Stream")

    with Cluster("4. Projections"):
        projector1 = Lambda("Views")
        projector2 = Lambda("Analyzer")

    with Cluster("5. Read Models"):
        view = DynamodbTable("View")
        s3 = S3("S3")

    user >> Edge(label="1. POST /dispenses") >> api
    api >> Edge(label="2. Invoke") >> cmd_lambda
    cmd_lambda >> Edge(label="3. Dispense:Started") >> events
    events >> Edge(label="4. Stream") >> publisher
    publisher >> Edge(label="5. Publish") >> stream
    stream >> Edge(label="6. Consume") >> projector1
    stream >> Edge(label="6. Consume") >> projector2
    projector1 >> Edge(label="7. Update") >> view
    projector2 >> Edge(label="7. Upload") >> s3

print("✅ Generated: aws_event_flow.png")

print("\n📊 Diagrams generated successfully!")
print("\nFiles created:")
print("  - aws_architecture.png (Main architecture)")
print("  - aws_architecture_detailed.png (Detailed with IAM)")
print("  - aws_event_flow.png (Event flow sequence)")
print("\n📝 To install dependencies:")
print("  pip install diagrams")
