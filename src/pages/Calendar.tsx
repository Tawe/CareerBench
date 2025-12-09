import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
import type { CalendarEvent, Reminder } from "../commands/types";
import "./Calendar.css";

export default function Calendar() {
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentDate, setCurrentDate] = useState(new Date());
  const [selectedDate, setSelectedDate] = useState<Date | null>(null);
  const [selectedEvents, setSelectedEvents] = useState<CalendarEvent[]>([]);
  const [reminders, setReminders] = useState<Reminder[]>([]);
  const [showReminderModal, setShowReminderModal] = useState(false);
  const [selectedEventForReminder, setSelectedEventForReminder] = useState<CalendarEvent | null>(null);

  const year = currentDate.getFullYear();
  const month = currentDate.getMonth();

  // Get first day of month and number of days
  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);
  const daysInMonth = lastDay.getDate();
  const startingDayOfWeek = firstDay.getDay();

  // Calculate date range for current month view
  const startDate = new Date(year, month, 1);
  const endDate = new Date(year, month + 1, 0);
  const startDateStr = startDate.toISOString().split('T')[0];
  const endDateStr = endDate.toISOString().split('T')[0];

  useEffect(() => {
    loadEvents();
    loadReminders();
    checkDueReminders();
    
    // Check for due reminders every minute
    const interval = setInterval(() => {
      checkDueReminders();
    }, 60000);
    
    return () => clearInterval(interval);
  }, [currentDate]);

  async function loadEvents() {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<CalendarEvent[]>("get_calendar_events", {
        startDate: startDateStr,
        endDate: endDateStr,
      });
      setEvents(result);
    } catch (err: any) {
      setError(err?.message || "Failed to load calendar events");
    } finally {
      setIsLoading(false);
    }
  }

  async function loadReminders() {
    try {
      const result = await invoke<Reminder[]>("get_reminders", {
        startDate: startDateStr,
        endDate: endDateStr,
        includeSent: false,
      });
      setReminders(result);
    } catch (err: any) {
      console.error("Failed to load reminders:", err);
    }
  }

  async function checkDueReminders() {
    try {
      const dueReminders = await invoke<Reminder[]>("get_due_reminders");
      for (const reminder of dueReminders) {
        if (reminder.id && !reminder.isSent) {
          // Send notification
          await sendNotification({
            title: "Reminder",
            body: reminder.message || `${reminder.reminderType} reminder`,
          });
          
          // Mark as sent
          await invoke("mark_reminder_sent", { reminderId: reminder.id });
          showToast("Reminder notification sent", "success");
        }
      }
    } catch (err: any) {
      console.error("Failed to check due reminders:", err);
    }
  }

  async function handleCreateReminder(event: CalendarEvent, reminderDate: string, message: string) {
    try {
      await invoke("create_reminder", {
        applicationId: event.applicationId,
        eventId: event.id,
        reminderType: event.eventType,
        reminderDate: reminderDate,
        message: message,
      });
      showToast("Reminder created successfully", "success");
      setShowReminderModal(false);
      setSelectedEventForReminder(null);
      loadReminders();
    } catch (err: any) {
      showToast(err?.message || "Failed to create reminder", "error");
    }
  }

  function getEventsForDate(date: Date): CalendarEvent[] {
    const dateStr = date.toISOString().split('T')[0];
    return events.filter(event => {
      const eventDate = new Date(event.eventDate).toISOString().split('T')[0];
      return eventDate === dateStr;
    });
  }

  function handleDateClick(date: Date) {
    setSelectedDate(date);
    setSelectedEvents(getEventsForDate(date));
  }

  function handlePreviousMonth() {
    setCurrentDate(new Date(year, month - 1, 1));
  }

  function handleNextMonth() {
    setCurrentDate(new Date(year, month + 1, 1));
  }

  function handleToday() {
    setCurrentDate(new Date());
  }

  async function handleExportEvent(event: CalendarEvent) {
    try {
      // For interview events, generate ICS file
      if (event.eventType === "InterviewScheduled" || event.eventType === "InterviewCompleted") {
        const title = event.title || `${event.jobTitle || "Interview"} at ${event.company || "Company"}`;
        const startTime = new Date(event.eventDate).toISOString();
        const endTime = new Date(new Date(event.eventDate).getTime() + 60 * 60 * 1000).toISOString(); // 1 hour default
        
        const icsContent = await invoke<string>("sync_interview_to_calendar", {
          applicationId: event.applicationId,
          eventId: event.id,
          title: title,
          startTime: startTime,
          endTime: endTime,
          location: null,
          notes: event.details || undefined,
        });

        const fileName = `interview-${event.company || "event"}-${new Date(event.eventDate).toISOString().split('T')[0]}.ics`;
        const filePath = await save({
          defaultPath: fileName,
          filters: [{
            name: 'ICS',
            extensions: ['ics']
          }]
        });

        if (filePath) {
          await writeTextFile(filePath, icsContent);
          showToast("Calendar event exported successfully", "success");
        }
      } else {
        showToast("Export only available for interview events", "info");
      }
    } catch (err: any) {
      showToast(err?.message || "Failed to export event", "error");
    }
  }

  const monthNames = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
  ];

  const dayNames = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

  if (isLoading) {
    return (
      <div className="calendar-page">
        <div className="calendar-header">
          <LoadingSkeleton width="200px" height="2rem" />
        </div>
        <div className="calendar-grid">
          <LoadingSkeleton variant="card" width="100%" height="400px" />
        </div>
      </div>
    );
  }

  return (
    <div className="calendar-page">
      <div className="calendar-header">
        <h1>Calendar</h1>
        <div className="calendar-controls">
          <button onClick={handlePreviousMonth} aria-label="Previous month">←</button>
          <button onClick={handleToday}>Today</button>
          <button onClick={handleNextMonth} aria-label="Next month">→</button>
          <h2>{monthNames[month]} {year}</h2>
        </div>
      </div>

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="calendar-layout">
        <div className="calendar-grid">
          {/* Day headers */}
          {dayNames.map(day => (
            <div key={day} className="calendar-day-header">
              {day}
            </div>
          ))}

          {/* Empty cells for days before month starts */}
          {Array.from({ length: startingDayOfWeek }).map((_, i) => (
            <div key={`empty-${i}`} className="calendar-day empty"></div>
          ))}

          {/* Days of the month */}
          {Array.from({ length: daysInMonth }).map((_, i) => {
            const day = i + 1;
            const date = new Date(year, month, day);
            const dayEvents = getEventsForDate(date);
            const isToday = date.toDateString() === new Date().toDateString();
            const isSelected = selectedDate?.toDateString() === date.toDateString();

            return (
              <div
                key={day}
                className={`calendar-day ${isToday ? "today" : ""} ${isSelected ? "selected" : ""} ${dayEvents.length > 0 ? "has-events" : ""}`}
                onClick={() => handleDateClick(date)}
              >
                <div className="day-number">{day}</div>
                {dayEvents.length > 0 && (
                  <div className="day-events">
                    {dayEvents.slice(0, 3).map((event, idx) => (
                      <div
                        key={idx}
                        className={`event-dot ${event.eventType.toLowerCase()}`}
                        title={`${event.title || event.eventType}: ${event.jobTitle || ""} at ${event.company || ""}`}
                      />
                    ))}
                    {dayEvents.length > 3 && (
                      <div className="event-more">+{dayEvents.length - 3}</div>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {selectedDate && (
          <div className="calendar-sidebar">
            <div className="sidebar-header">
              <h3>{selectedDate.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' })}</h3>
              <button onClick={() => setSelectedDate(null)} aria-label="Close">×</button>
            </div>
            <div className="sidebar-events">
              {selectedEvents.length === 0 ? (
                <p className="no-events">No events scheduled for this day</p>
              ) : (
                selectedEvents.map((event, idx) => (
                  <div key={idx} className="event-card">
                    <div className="event-header">
                      <h4>{event.title || event.eventType}</h4>
                      <div className="event-actions">
                        {(event.eventType === "InterviewScheduled" || event.eventType === "InterviewCompleted") && (
                          <>
                            <button
                              onClick={() => handleExportEvent(event)}
                              className="export-button"
                              aria-label="Export to calendar"
                              title="Export to calendar"
                            >
                              📅
                            </button>
                            <button
                              onClick={() => {
                                setSelectedEventForReminder(event);
                                setShowReminderModal(true);
                              }}
                              className="reminder-button"
                              aria-label="Set reminder"
                              title="Set reminder"
                            >
                              ⏰
                            </button>
                          </>
                        )}
                      </div>
                    </div>
                    <p className="event-company">
                      {event.jobTitle || "Untitled"} at {event.company || "Unknown Company"}
                    </p>
                    {event.details && (
                      <p className="event-details">{event.details}</p>
                    )}
                    {event.nextActionNote && (
                      <p className="event-note">📝 {event.nextActionNote}</p>
                    )}
                    <div className="event-time">
                      {new Date(event.eventDate).toLocaleTimeString('en-US', {
                        hour: 'numeric',
                        minute: '2-digit'
                      })}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {showReminderModal && selectedEventForReminder && (
          <ReminderModal
            event={selectedEventForReminder}
            onClose={() => {
              setShowReminderModal(false);
              setSelectedEventForReminder(null);
            }}
            onCreate={handleCreateReminder}
          />
        )}
      </div>
    </div>
  );
}

function ReminderModal({
  event,
  onClose,
  onCreate,
}: {
  event: CalendarEvent;
  onClose: () => void;
  onCreate: (event: CalendarEvent, reminderDate: string, message: string) => void;
}) {
  const [reminderDate, setReminderDate] = useState("");
  const [reminderTime, setReminderTime] = useState("");
  const [message, setMessage] = useState("");

  useEffect(() => {
    // Default to 1 hour before event
    const eventDate = new Date(event.eventDate);
    const reminderDateTime = new Date(eventDate.getTime() - 60 * 60 * 1000);
    setReminderDate(reminderDateTime.toISOString().split('T')[0]);
    setReminderTime(reminderDateTime.toTimeString().slice(0, 5));
    setMessage(`Reminder: ${event.title || event.eventType} at ${event.company || "Company"}`);
  }, [event]);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const reminderDateTime = `${reminderDate}T${reminderTime}:00`;
    onCreate(event, reminderDateTime, message);
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>Create Reminder</h3>
          <button onClick={onClose} aria-label="Close">×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Event</label>
            <p className="form-readonly">{event.title || event.eventType} - {event.company}</p>
          </div>
          <div className="form-group">
            <label htmlFor="reminder-date">Reminder Date</label>
            <input
              id="reminder-date"
              type="date"
              value={reminderDate}
              onChange={(e) => setReminderDate(e.target.value)}
              required
            />
          </div>
          <div className="form-group">
            <label htmlFor="reminder-time">Reminder Time</label>
            <input
              id="reminder-time"
              type="time"
              value={reminderTime}
              onChange={(e) => setReminderTime(e.target.value)}
              required
            />
          </div>
          <div className="form-group">
            <label htmlFor="reminder-message">Message</label>
            <textarea
              id="reminder-message"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              rows={3}
              required
            />
          </div>
          <div className="modal-actions">
            <button type="button" onClick={onClose}>Cancel</button>
            <button type="submit">Create Reminder</button>
          </div>
        </form>
      </div>
    </div>
  );
}


