import { BrowserRouter, Routes, Route } from "react-router-dom";
import { lazy, Suspense } from "react";
import Layout from "./components/Layout";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { LoadingSkeleton } from "./components/LoadingSkeleton";

const Dashboard = lazy(() => import("./pages/Dashboard"));
const Profile = lazy(() => import("./pages/Profile"));
const Jobs = lazy(() => import("./pages/Jobs"));
const Applications = lazy(() => import("./pages/Applications"));
const Calendar = lazy(() => import("./pages/Calendar"));
const Settings = lazy(() => import("./pages/Settings"));
const Learning = lazy(() => import("./pages/Learning"));
const Recruiters = lazy(() => import("./pages/Recruiters"));
const Companies = lazy(() => import("./pages/Companies"));

function App() {
  return (
    <BrowserRouter>
      <Layout>
        <Suspense fallback={<LoadingSkeleton variant="card" lines={3} />}>
          <Routes>
            <Route path="/" element={
              <ErrorBoundary>
                <Dashboard />
              </ErrorBoundary>
            } />
            <Route path="/profile" element={
              <ErrorBoundary>
                <Profile />
              </ErrorBoundary>
            } />
            <Route path="/jobs" element={
              <ErrorBoundary>
                <Jobs />
              </ErrorBoundary>
            } />
            <Route path="/applications" element={
              <ErrorBoundary>
                <Applications />
              </ErrorBoundary>
            } />
            <Route path="/calendar" element={
              <ErrorBoundary>
                <Calendar />
              </ErrorBoundary>
            } />
            <Route path="/learning" element={
              <ErrorBoundary>
                <Learning />
              </ErrorBoundary>
            } />
            <Route path="/recruiters" element={
              <ErrorBoundary>
                <Recruiters />
              </ErrorBoundary>
            } />
            <Route path="/companies" element={
              <ErrorBoundary>
                <Companies />
              </ErrorBoundary>
            } />
            <Route path="/settings" element={
              <ErrorBoundary>
                <Settings />
              </ErrorBoundary>
            } />
          </Routes>
        </Suspense>
      </Layout>
    </BrowserRouter>
  );
}

export default App;

