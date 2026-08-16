{{/*
Expand the name of the chart.
*/}}
{{- define "circus.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "circus.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{- define "circus.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "circus.labels" -}}
helm.sh/chart: {{ include "circus.chart" . }}
{{ include "circus.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "circus.selectorLabels" -}}
app.kubernetes.io/name: {{ include "circus.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "circus.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "circus.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{/*
Name of the secret holding the JWT signing secret.
*/}}
{{- define "circus.jwtSecretName" -}}
{{- if .Values.jwt.existingSecret -}}
{{- .Values.jwt.existingSecret -}}
{{- else -}}
{{- printf "%s-jwt" (include "circus.fullname" .) -}}
{{- end -}}
{{- end -}}

{{/*
Name of the secret holding bootstrap admin credentials.
*/}}
{{- define "circus.bootstrapSecretName" -}}
{{- if .Values.bootstrapAdmin.existingSecret -}}
{{- .Values.bootstrapAdmin.existingSecret -}}
{{- else -}}
{{- printf "%s-bootstrap" (include "circus.fullname" .) -}}
{{- end -}}
{{- end -}}

{{/*
Name of the secret holding DATABASE_URL, when using the bundled postgresql
subchart. Bitnami's chart names its generated secret <release>-postgresql.
*/}}
{{- define "circus.postgresqlFullname" -}}
{{- printf "%s-postgresql" .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
