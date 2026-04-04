export interface EndpointDoc {
  path: string;
  method: string;
  status_code: number;
  title: string;
  summary: string;
  markdown: string;
  limitations: string;
}

export interface GeneratedApiDocs {
  title: string;
  introduction: string;
  endpoints: EndpointDoc[];
}

export interface DocsGenerateErrorBody {
  error: string;
}
