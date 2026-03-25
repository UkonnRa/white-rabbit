export interface Journal {
  id: string;
  name: string;
  description: string;
  tags: string[];
  created_at: string | null;
  last_modified_at: string | null;
  archived_at: string | null;
}

export interface CreateJournalRequest {
  name: string;
  description?: string;
  tags?: string[];
}

export interface UpdateJournalRequest {
  name?: string;
  description?: string;
  tags?: string[];
}

export interface JournalFilter {
  id?: string;
  name?: string;
  tag?: string;
  fullText?: string;
}

export interface JournalFormData {
  name: string;
  description: string;
  tags: string[];
}
