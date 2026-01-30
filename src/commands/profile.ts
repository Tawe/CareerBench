/**
 * User profile command types
 */

export interface UserProfile {
  id?: number;
  fullName: string;
  headline?: string;
  location?: string;
  summary?: string;
  currentRoleTitle?: string;
  currentCompany?: string;
  seniority?: string;
  openToRoles?: string;
  createdAt?: string;
  updatedAt?: string;
}

export interface Experience {
  id?: number;
  company: string;
  title: string;
  location?: string;
  startDate?: string;
  endDate?: string;
  isCurrent: boolean;
  description?: string;
  achievements?: string;
  techStack?: string;
}

export interface Skill {
  id?: number;
  name: string;
  category?: string;
  selfRating?: number;
  priority?: string;
  yearsExperience?: number;
  notes?: string;
}

export interface Education {
  id?: number;
  institution: string;
  degree?: string;
  fieldOfStudy?: string;
  startDate?: string;
  endDate?: string;
  grade?: string;
  description?: string;
}

export interface Certification {
  id?: number;
  name: string;
  issuingOrganization?: string;
  issueDate?: string;
  expirationDate?: string;
  credentialId?: string;
  credentialUrl?: string;
}

export interface PortfolioItem {
  id?: number;
  title: string;
  url?: string;
  description?: string;
  role?: string;
  techStack?: string;
  highlighted: boolean;
}

export interface UserProfileData {
  profile: UserProfile | null;
  experience: Experience[];
  skills: Skill[];
  education: Education[];
  certifications: Certification[];
  portfolio: PortfolioItem[];
}

export interface ProfileCommands {
  get_user_profile_data: {
    args: [];
    return: UserProfileData;
  };
  save_user_profile_data: {
    args: [data: UserProfileData];
    return: UserProfileData;
  };
  generate_profile_summary: {
    args: [];
    return: string;
  };
  extract_skills_from_experience: {
    args: [];
    return: string[];
  };
  rewrite_portfolio_description: {
    args: [portfolioId: number, description: string];
    return: string;
  };
}