Class TestingClassKeywordInheritance.c [Not ProcedureBlock, Language = objectscript, inheritance = right]
{

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
    Method newVarChange() [ProcedureBlock=1] {
        set x = 2
        w x
    }

    

    ClassMethod multiplePubVarDefs()  {
        set y = 24 
        w x
        d ..testingIndentation()
    }

    Method testingIndentation() {
        set x = 3
        w y

    }
}

