Class MyApp.Main.SampleClass Extends %RegisteredObject
{

Parameter CONSTANTMESSAGE [Internal] = "Hello world!" ;
Property VariableMessage As %String [ InitialExpression = "How are you?"];

ClassMethod HelloWorld() As %String [ Language = objectscript ]
    {
        Set x=..#CONSTANTMESSAGE
        Return x
    }

Method WriteIt() [ Language = objectscript, ServerOnly = 1]
    {
        Set count=..MessageCount
        For i=1:1:count {
            Write !,..#CONSTANTMESSAGE," ",..VariableMessage
        }
    }

ClassMethod %FromObject(source As %RegisteredObject, useadaptor As %Boolean = 1) As %DynamicObject {
if '$ISOBJECT(source) Return {}

    // %JSON.Adaptor: export via stream to avoid
    If useadaptor && source.%Extends("%JSON.Adaptor") {
       	set stream = ##class(%Stream.TmpCharacter).%New()
        $$$ThrowOnError(source.%JSONExportToStream(stream))
        Return ##class(%DynamicObject).%FromJSON(stream)
    }
}
}
