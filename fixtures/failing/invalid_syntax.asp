<%
' This file contains intentionally invalid ASP Classic syntax
' that should fail to parse

' Unclosed ASP tag (missing %>)
<% 

' Mismatched quotes in string
Response.Write "Hello, World!

' Invalid statement syntax
For i = 1 To 10 End

' Incomplete Response.Write
Response.Write 

' Unclosed tag with invalid contents
<%=incomplete expression