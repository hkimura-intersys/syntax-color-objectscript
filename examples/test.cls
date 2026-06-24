Class TestingClassKeywordInheritance.c [Not ProcedureBlock, Language = objectscript, inheritance = right]
{

    Xdata test [mimetype = "text/markdown"]
    {
        
    }
    XData SampleAPI [mimetype = "application/yaml"] 
    {
        swagger: "2.0"
        info:
        title: Sample API
        description: API description in Markdown.
        version: 1.0.0
        host: api.example.com
        basePath: /v1
        schemes:
        - https
        paths:
        /users:
            get:
            summary: Returns a list of users.
            description: Optional extended description in Markdown.
            produces:
                - application/json
            responses:
                200:
                description: OK
    }

    XData SampleTest [mimetype = "application/json"] 
    {
        {
        "name" : "hello" , 
        "okay" : 1,
        }
    }

    Method newVarChange() [ProcedureBlock=1] {
        set x = 2
        w x
    }

    ClassMethod multiplePubVarDefs()  {
        set y = 24 
        w x
        d ..testingIndentation()
    }

    ClassMethod testingIndentation() {
        set x = 3
        w y

    }
}

