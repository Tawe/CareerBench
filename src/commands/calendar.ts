/**
 * Calendar command types
 */

export interface CalendarEvent {
  id?: number;
  applicationId: number;
  jobTitle?: string;
  company?: string;
  eventType: string;
  eventDate: string;
  title?: string;
  details?: string;
  nextActionDate?: string;
  nextActionNote?: string;
}

export interface CalendarCommands {
  get_calendar_events: {
    args: [startDate: string, endDate: string];
    return: CalendarEvent[];
  };
  get_events_for_date: {
    args: [date: string];
    return: CalendarEvent[];
  };
  sync_interview_to_calendar: {
    args: [
      applicationId: number,
      eventId: number | null,
      title: string,
      startTime: string,
      endTime: string | null,
      location: string | null,
      notes: string | null
    ];
    return: string; // ICS file content or sync confirmation
  };
}