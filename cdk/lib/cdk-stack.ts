import { join } from 'path';
import { RustFunction } from 'cargo-lambda-cdk';
import { EndpointType, LambdaRestApi } from 'aws-cdk-lib/aws-apigateway'
import { AttributeType, Table, BillingMode } from 'aws-cdk-lib/aws-dynamodb';
import { RemovalPolicy, Stack, StackProps } from "aws-cdk-lib";
import { Construct } from "constructs";


export class CdkStack extends Stack {
    constructor(scope: Construct, id: string, props?: StackProps) {
        super(scope, id, props);

        const dynamoTable = new Table(this, 'RustLambdaDemoTable', {
            partitionKey: { name: 'id', type: AttributeType.STRING },
            billingMode: BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });

        const lmabdaHandler = new RustFunction(this, 'RustLambdaDemoFunction', {
            // Path to the root directory.
            manifestPath: join(__dirname, '..', '..'),
            environment: {
                "DYNAMO_TABLE_NAME": dynamoTable.tableName
            }
        });

        dynamoTable.grantFullAccess(lmabdaHandler)

        new LambdaRestApi(this, 'RustLambdaDemoAPIGateway', {
            handler: lmabdaHandler,
            endpointTypes: [EndpointType.REGIONAL]
        });
    }
}