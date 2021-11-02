## slsswag

CLI tool that generates the Serverless framework project from the Swagger (OpenAPI specification) file in milliseconds.

[Blog post with more information](https://almirzulic.com/posts/migrate-swagger-to-serverless/)

**Usage:**

{{< highlight bash >}}
slsswag path/to/swagger/file.yml platform` 
{{< /highlight >}}

{{< highlight bash >}}
slsswag sample/swagger.yml nodejs
{{< /highlight >}}

{{< highlight bash >}}
slsswag sample/swagger.yml csharp
{{< /highlight >}}


**Note**: Currently works only with yaml. JSON is planned to be added later.

Sample folder contains [petstore](https://petstore.swagger.io) swagger file. When you execute the command, it will create the output folder with the complete project ready to be deployed. At the time of writing this article, the project is still under heavy development, but the NodeJS example works.

The program parses a swagger file and converts it into a format acceptable by the documentation plugin. For NodeJS it will generate:

- Full serverless.yml with plugins, functions, and doc configuration
- package.json
- Function files with the code in it
- Tests for the functions (Jest)
- Documentation for each function file
- Models

After running the command, you only need to execute `npm install` and your service is ready to be deployed. 

1. `./slsswag sample/swagger.yml nodejs`
2. `cd output`
3. `npm i`
4. `sls deploy`

One more important note is that API Gateway documentation does not support `xml` and `example` tags. Therefore you need to remove it from the generated Swagger, if any. Removing those tags is on a roadmap, but you have to do it on your own for now.

## License
The source code for this project is released under the [MIT License](/LICENSE).