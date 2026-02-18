{{- define "resourcePrefix" -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- end }}
