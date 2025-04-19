<%
' Basic ASP Classic file for testing the parser
Dim greeting
greeting = "Hello, World!"

' Test a Response.Write statement
Response.Write greeting

' Test statement separator
Dim x, y : x = 10 : y = 20

' Test line continuation
Dim longString
longString = "This is a very long string that " & _
             "continues on the next line"
%>

<h1>ASP Classic Test Page</h1>

<%
' Another code block
If x > 5 Then
    Response.Write("<p>X is greater than 5</p>")
End If
%>

<%=greeting%>