/**
 * Applications command types
 */

export type ApplicationStatus =
  | "Saved"
  | "Draft"
  | "Applied"
  | "Interviewing"
  | "Offer"
  | "Rejected"
  | "Ghosted"
  | "Withdrawn";

export interface Application {
  id?: number;
  jobId: number;
  status: ApplicationStatus;
  channel?: string;
  priority?: "Low" | "Medium" | "High" | "Dream";
  dateSaved: string;
  dateApplied?: string;
  lastActivityDate?: string;
  nextActionDate?: string;
  nextActionNote?: string;
  notesSummary?: string;
  contactName?: string;
  contactEmail?: string;
  contactLinkedin?: string;
  locationOverride?: string;
  offerCompensation?: string;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface ApplicationSummary {
  id: number;
  jobId: number;
  jobTitle?: string;
  company?: string;
  status: ApplicationStatus;
  priority?: "Low" | "Medium" | "High" | "Dream";
  dateSaved: string;
  dateApplied?: string;
  lastActivityDate?: string;
}

export interface ApplicationEvent {
  id?: number;
  applicationId: number;
  eventType: string;
  eventDate: string;
  fromStatus?: string;
  toStatus?: string;
  title?: string;
  details?: string;
  createdAt: string;
}

export interface ApplicationDetail {
  application: Application;
  events: ApplicationEvent[];
}

export interface CreateApplicationInput {
  jobId: number;
  status?: ApplicationStatus;
  channel?: string;
  priority?: "Low" | "Medium" | "High" | "Dream";
}

export interface UpdateApplicationInput {
  status?: ApplicationStatus;
  channel?: string;
  priority?: "Low" | "Medium" | "High" | "Dream";
  dateApplied?: string;
  nextActionDate?: string;
  nextActionNote?: string;
  notesSummary?: string;
  contactName?: string;
  contactEmail?: string;
  contactLinkedin?: string;
  locationOverride?: string;
  offerCompensation?: string;
}

export interface AddEventInput {
  applicationId: number;
  eventType: string;
  eventDate: string;
  fromStatus?: string;
  toStatus?: string;
  title?: string;
  details?: string;
}

export interface ApplicationCommands {
  create_application: {
    args: [input: CreateApplicationInput];
    return: Application;
  };
  update_application: {
    args: [id: number, input: UpdateApplicationInput];
    return: Application;
  };
  get_applications: {
    args: [options?: { status?: ApplicationStatus | null; jobId?: number | null; activeOnly?: boolean }];
    return: ApplicationSummary[];
  };
  get_application_detail: {
    args: [id: number];
    return: ApplicationDetail;
  };
  add_application_event: {
    args: [input: AddEventInput];
    return: ApplicationEvent;
  };
  archive_application: {
    args: [id: number];
    return: Application;
  };
}