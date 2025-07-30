import React, { useState } from "react";
import Signup from "./pages/Signup";
import VerifyOTP from "./pages/VerifyOTP";
import Login from "./pages/Login";
import Dashboard from "./components/Dashboard";

function App() {
  const [currentPage, setCurrentPage] = useState<'signup' | 'verify-otp' | 'login' | 'dashboard'>('signup');
  const [userEmail, setUserEmail] = useState('');
  const [userInfo, setUserInfo] = useState<any>(null);

  const handleSignupSuccess = (email: string) => {
    setUserEmail(email);
    setCurrentPage('verify-otp');
  };

  const handleOTPVerified = () => {
    setCurrentPage('login');
  };

  const handleLoginSuccess = (token: string, user: any) => {
    setUserInfo(user);
    setCurrentPage('dashboard');
  };

  const handleBackToSignup = () => {
    setCurrentPage('signup');
    setUserEmail('');
  };

  const handleGoToLogin = () => {
    setCurrentPage('login');
  };

  const handleGoToSignup = () => {
    setCurrentPage('signup');
    setUserEmail('');
    setUserInfo(null);
  };

  const handleLogout = () => {
    localStorage.removeItem('auth_token');
    setUserInfo(null);
    setCurrentPage('login');
  };

  return (
    <div>
      {currentPage === 'signup' && (
        <Signup 
          onSignupSuccess={handleSignupSuccess}
          onGoToLogin={handleGoToLogin}
        />
      )}
      {currentPage === 'verify-otp' && (
        <VerifyOTP 
          defaultEmail={userEmail} 
          onBackToSignup={handleBackToSignup}
          onOTPVerified={handleOTPVerified}
        />
      )}
      {currentPage === 'login' && (
        <Login 
          onLoginSuccess={handleLoginSuccess}
          onGoToSignup={handleGoToSignup}
        />
      )}
      {currentPage === 'dashboard' && (
        <Dashboard 
          userInfo={userInfo}
          onLogout={handleLogout}
        />
      )}
    </div>
  );
}

export default App;
