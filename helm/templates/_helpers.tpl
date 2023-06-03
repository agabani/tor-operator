{{/*
Expand the name of the chart.
*/}}
{{- define "tor-operator.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "tor-operator.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "tor-operator.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "tor-operator.labels" -}}
helm.sh/chart: {{ include "tor-operator.chart" . }}
{{ include "tor-operator.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "tor-operator.selectorLabels" -}}
app.kubernetes.io/name: {{ include "tor-operator.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the cluster role to use
*/}}
{{- define "tor-operator.clusterRoleName" -}}
{{- if .Values.clusterRole.create }}
{{- default (include "tor-operator.fullname" .) .Values.clusterRole.name }}
{{- else }}
{{- default "default" .Values.clusterRole.name }}
{{- end }}
{{- end }}

{{/*
Create the name of the cluster role binding to use
*/}}
{{- define "tor-operator.clusterRoleBindingName" -}}
{{- if .Values.clusterRoleBinding.create }}
{{- default (include "tor-operator.fullname" .) .Values.clusterRoleBinding.name }}
{{- else }}
{{- default "default" .Values.clusterRoleBinding.name }}
{{- end }}
{{- end }}

{{/*
Create the name of the role to use
*/}}
{{- define "tor-operator.roleName" -}}
{{- if .Values.role.create }}
{{- default (include "tor-operator.fullname" .) .Values.role.name }}
{{- else }}
{{- default "default" .Values.role.name }}
{{- end }}
{{- end }}

{{/*
Create the name of the role binding to use
*/}}
{{- define "tor-operator.roleBindingName" -}}
{{- if .Values.roleBinding.create }}
{{- default (include "tor-operator.fullname" .) .Values.roleBinding.name }}
{{- else }}
{{- default "default" .Values.roleBinding.name }}
{{- end }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "tor-operator.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "tor-operator.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}
