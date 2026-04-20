export interface EndpointDoc {
  path: string;
  method: string;
  status_code: number;
  title: string;
  summary: string;
  observed_request: string[];
  observed_response: string[];
  markdown: string;
  limitations: string[];
}

export interface GeneratedApiDocs {
  title: string;
  introduction: string;
  highlights: string[];
  endpoints: EndpointDoc[];
}

export interface DocsGenerateErrorBody {
  error: string;
}
