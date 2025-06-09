export interface GetInstructorsRequest {
  id: string;
}

export interface GetInstructorsResponse {
  pages: {
    id: string
    name: string
    age: number
  }
}