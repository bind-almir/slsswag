# slsswag

The idea comes from several projects that I was migrating to serverless architecture using Serverless framework. There is an existing OpenAPI specification, but there is no good way to migrate it to Serverless. You have to define all the endpoints over and over again which is a kind of redundant work. 

Status: Experimental
Note: Currently works only with yaml. JSON is planned to be added later.

Usage:
- `slsswag sample/swagger.yml nodejs`
- `slsswag sample/swagger.yml csharp`


Sample folder contains [petstore swagger file](https://petstore.swagger.io/). Once the command is executed, the output folder will be created with the complete project ready to be deployed. The project is still under development but the NodeJS example works.

1. `cargo run sample/swagger.yml nodejs`
2. `cd output`
3. `npm i`
4. `sls deploy`

Suppose you have your legacy application with the OpenAPI specification. If you plan to migrate to the Serverless framework and run on AWS, export the file, then run the command against your file. 