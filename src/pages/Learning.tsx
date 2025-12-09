import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
import type {
  SkillGap,
  LearningPlan,
  LearningTrack,
  LearningTask,
  LearningResource,
} from "../commands/types";
import "./Learning.css";

export default function Learning() {
  const [skillGaps, setSkillGaps] = useState<SkillGap[]>([]);
  const [learningPlans, setLearningPlans] = useState<LearningPlan[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [showCreatePlan, setShowCreatePlan] = useState(false);
  const [selectedPlan, setSelectedPlan] = useState<LearningPlan | null>(null);
  const [selectedTrack, setSelectedTrack] = useState<LearningTrack | null>(null);
  const [tracks, setTracks] = useState<LearningTrack[]>([]);
  const [tasks, setTasks] = useState<LearningTask[]>([]);
  const [resources, setResources] = useState<LearningResource[]>([]);

  useEffect(() => {
    loadLearningPlans();
  }, []);

  async function loadLearningPlans() {
    setIsLoading(true);
    try {
      const plans = await invoke<LearningPlan[]>("get_learning_plans", { status: null });
      setLearningPlans(plans);
    } catch (err: any) {
      showToast(err?.message || "Failed to load learning plans", "error");
    } finally {
      setIsLoading(false);
    }
  }

  async function handleAnalyzeSkillGaps(includeAllJobs: boolean) {
    setIsAnalyzing(true);
    try {
      const gaps = await invoke<SkillGap[]>("analyze_skill_gaps", {
        jobId: null,
        includeAllJobs: includeAllJobs,
      });
      setSkillGaps(gaps);
      setShowCreatePlan(true);
    } catch (err: any) {
      showToast(err?.message || "Failed to analyze skill gaps", "error");
    } finally {
      setIsAnalyzing(false);
    }
  }

  async function handleCreatePlan(title: string, description: string) {
    try {
      const planId = await invoke<number>("create_learning_plan", {
        title,
        description: description || null,
        targetJobId: null,
        skillGaps: skillGaps,
        estimatedDurationDays: null,
      });
      
      // Generate learning content using AI
      showToast("Generating learning tracks and tasks with AI...", "info");
      try {
        await invoke("generate_learning_content", {
          learningPlanId: planId,
          skillGaps: skillGaps,
        });
        showToast("Learning plan created with AI-generated tracks and tasks!", "success");
      } catch (aiErr: any) {
        // If AI generation fails, still show success for plan creation
        showToast("Learning plan created. AI generation failed - you can add tracks manually.", "warning");
      }
      
      setShowCreatePlan(false);
      setSkillGaps([]);
      loadLearningPlans();
    } catch (err: any) {
      showToast(err?.message || "Failed to create learning plan", "error");
    }
  }

  async function loadPlanDetails(plan: LearningPlan) {
    setSelectedPlan(plan);
    try {
      const planTracks = await invoke<LearningTrack[]>("get_learning_tracks", {
        learningPlanId: plan.id,
      });
      setTracks(planTracks);
    } catch (err: any) {
      showToast(err?.message || "Failed to load learning tracks", "error");
    }
  }

  async function loadTrackTasks(track: LearningTrack) {
    setSelectedTrack(track);
    try {
      const trackTasks = await invoke<LearningTask[]>("get_learning_tasks", {
        learningTrackId: track.id,
      });
      setTasks(trackTasks);
    } catch (err: any) {
      showToast(err?.message || "Failed to load learning tasks", "error");
    }
  }

  async function handleCompleteTask(taskId: number, completed: boolean) {
    try {
      await invoke("complete_learning_task", { taskId, completed });
      if (selectedTrack) {
        loadTrackTasks(selectedTrack);
      }
      showToast(completed ? "Task marked as completed" : "Task marked as incomplete", "success");
    } catch (err: any) {
      showToast(err?.message || "Failed to update task", "error");
    }
  }

  async function handleDeletePlan(planId: number) {
    if (!confirm("Are you sure you want to delete this learning plan?")) return;
    try {
      await invoke("delete_learning_plan", { planId });
      showToast("Learning plan deleted", "success");
      setSelectedPlan(null);
      loadLearningPlans();
    } catch (err: any) {
      showToast(err?.message || "Failed to delete learning plan", "error");
    }
  }

  if (isLoading) {
    return (
      <div className="learning-page">
        <div className="learning-header">
          <LoadingSkeleton width="200px" height="2rem" />
        </div>
        <div className="learning-content">
          <LoadingSkeleton variant="card" width="100%" height="400px" />
        </div>
      </div>
    );
  }

  return (
    <div className="learning-page">
      <div className="learning-header">
        <h1>Learning Plans</h1>
        <div className="learning-actions">
          <button
            onClick={() => handleAnalyzeSkillGaps(false)}
            disabled={isAnalyzing}
            className="btn-primary"
          >
            {isAnalyzing ? "Analyzing..." : "Analyze Skill Gaps (Current Job)"}
          </button>
          <button
            onClick={() => handleAnalyzeSkillGaps(true)}
            disabled={isAnalyzing}
            className="btn-secondary"
          >
            {isAnalyzing ? "Analyzing..." : "Analyze All Jobs"}
          </button>
        </div>
      </div>

      {skillGaps.length > 0 && (
        <div className="skill-gaps-panel">
          <h2>Skill Gap Analysis</h2>
          <div className="skill-gaps-list">
            {skillGaps.slice(0, 10).map((gap, idx) => (
              <div key={idx} className={`skill-gap-item ${gap.priority}`}>
                <div className="skill-gap-header">
                  <span className="skill-name">{gap.skill}</span>
                  <span className={`priority-badge priority-${gap.priority}`}>
                    {gap.priority.toUpperCase()}
                  </span>
                </div>
                <div className="skill-gap-details">
                  <span>Found in {gap.frequency} job{gap.frequency !== 1 ? "s" : ""}</span>
                  {gap.user_has_skill ? (
                    <span className="has-skill">✓ You have this skill</span>
                  ) : (
                    <span className="missing-skill">✗ Skill gap identified</span>
                  )}
                </div>
              </div>
            ))}
          </div>
          {showCreatePlan && (
            <CreatePlanModal
              skillGaps={skillGaps}
              onClose={() => {
                setShowCreatePlan(false);
                setSkillGaps([]);
              }}
              onCreate={handleCreatePlan}
            />
          )}
        </div>
      )}

      <div className="learning-content">
        <div className="learning-plans-list">
          <h2>Your Learning Plans</h2>
          {learningPlans.length === 0 ? (
            <div className="empty-state">
              <p>No learning plans yet. Analyze skill gaps to create your first learning plan.</p>
            </div>
          ) : (
            learningPlans.map((plan) => (
              <div
                key={plan.id}
                className={`learning-plan-card ${selectedPlan?.id === plan.id ? "active" : ""}`}
                onClick={() => loadPlanDetails(plan)}
              >
                <div className="plan-header">
                  <h3>{plan.title}</h3>
                  <div className="plan-actions">
                    <span className={`status-badge status-${plan.status}`}>{plan.status}</span>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        plan.id && handleDeletePlan(plan.id);
                      }}
                      className="delete-button"
                      aria-label="Delete plan"
                    >
                      ×
                    </button>
                  </div>
                </div>
                {plan.description && <p className="plan-description">{plan.description}</p>}
                {plan.estimated_duration_days && (
                  <div className="plan-meta">
                    Estimated duration: {plan.estimated_duration_days} days
                  </div>
                )}
              </div>
            ))
          )}
        </div>

        {selectedPlan && (
          <div className="learning-plan-detail">
            <div className="detail-header">
              <h2>{selectedPlan.title}</h2>
              <button onClick={() => setSelectedPlan(null)} aria-label="Close">×</button>
            </div>
            {tracks.length === 0 ? (
              <div className="empty-state">
                <p>No learning tracks yet. Tracks will be generated when you create tasks.</p>
              </div>
            ) : (
              <div className="tracks-list">
                {tracks.map((track) => (
                  <div
                    key={track.id}
                    className={`track-card ${selectedTrack?.id === track.id ? "active" : ""}`}
                    onClick={() => loadTrackTasks(track)}
                  >
                    <h4>{track.title}</h4>
                    {track.description && <p>{track.description}</p>}
                    {track.skill_focus && (
                      <div className="skill-focus">Focus: {track.skill_focus}</div>
                    )}
                  </div>
                ))}
              </div>
            )}

            {selectedTrack && (
              <div className="tasks-panel">
                <h3>{selectedTrack.title}</h3>
                {tasks.length === 0 ? (
                  <div className="empty-state">
                    <p>No tasks yet for this track.</p>
                  </div>
                ) : (
                  <div className="tasks-list">
                    {tasks.map((task) => (
                      <div key={task.id} className={`task-item ${task.completed ? "completed" : ""}`}>
                        <div className="task-header">
                          <input
                            type="checkbox"
                            checked={task.completed}
                            onChange={(e) => task.id && handleCompleteTask(task.id, e.target.checked)}
                          />
                          <h4>{task.title}</h4>
                        </div>
                        {task.description && <p className="task-description">{task.description}</p>}
                        <div className="task-meta">
                          {task.estimated_hours && (
                            <span>Est. {task.estimated_hours} hours</span>
                          )}
                          {task.resource_url && (
                            <a href={task.resource_url} target="_blank" rel="noopener noreferrer">
                              View Resource
                            </a>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function CreatePlanModal({
  skillGaps,
  onClose,
  onCreate,
}: {
  skillGaps: SkillGap[];
  onClose: () => void;
  onCreate: (title: string, description: string) => void;
}) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!title.trim()) {
      showToast("Please enter a title", "error");
      return;
    }
    onCreate(title, description);
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>Create Learning Plan</h3>
          <button onClick={onClose} aria-label="Close">×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Title *</label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="e.g., Full Stack Development Skills"
              required
            />
          </div>
          <div className="form-group">
            <label>Description</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              placeholder="Optional description of your learning goals"
            />
          </div>
          <div className="modal-actions">
            <button type="button" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn-primary">Create Plan</button>
          </div>
        </form>
      </div>
    </div>
  );
}
