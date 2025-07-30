// src/pages/VerifyOTP.tsx
import { useState, useEffect } from "react";

interface VerifyOTPProps {
  defaultEmail?: string;
  onBackToSignup?: () => void;
  onOTPVerified?: () => void;
}

const VerifyOTP = ({ defaultEmail = "", onBackToSignup, onOTPVerified }: VerifyOTPProps) => {
  const [email, setEmail] = useState(defaultEmail);
  const [otp, setOtp] = useState("");
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    setEmail(defaultEmail);
  }, [defaultEmail]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setMessage("");
    setError("");

    try {
      const res = await fetch("http://localhost:8080/verify-otp", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ email, otp }),
      });

      const data = await res.json();

      if (res.ok) {
        setMessage("OTP verified successfully! Redirecting to login...");
        setOtp("");
        setTimeout(() => {
          if (onOTPVerified) {
            onOTPVerified();
          }
        }, 1500);
      } else {
        setError(data.error || "Verification failed.");
      }
    } catch (err) {
      setError("Something went wrong. Try again.");
    }
  };

  const handleResendOTP = async () => {
    if (!email) {
      setError("Please enter your email first.");
      return;
    }

    try {
      const res = await fetch("http://localhost:8080/send-otp", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ email }),
      });

      if (res.ok) {
        setMessage("OTP sent successfully! Check your email.");
        setError("");
      } else {
        const data = await res.json();
        setError(data.error || "Failed to send OTP.");
      }
    } catch (err) {
      setError("Something went wrong. Try again.");
    }
  };

  return (
    <div className="min-h-screen bg-gray-100 flex items-center justify-center px-4">
      <div className="bg-white p-8 rounded-lg shadow-md w-full max-w-sm">
        <h2 className="text-2xl font-bold text-center mb-4">Verify OTP</h2>

        {message && <p className="text-green-600 text-center mb-2">{message}</p>}
        {error && <p className="text-red-600 text-center mb-2">{error}</p>}

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">Email</label>
            <input
              type="email"
              className="w-full border rounded px-3 py-2 focus:outline-none focus:ring focus:border-blue-500"
              placeholder="Enter your email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">OTP</label>
            <input
              type="text"
              className="w-full border rounded px-3 py-2 focus:outline-none focus:ring focus:border-blue-500"
              placeholder="Enter the 6-digit OTP"
              value={otp}
              onChange={(e) => setOtp(e.target.value)}
              maxLength={6}
              required
            />
          </div>

          <button
            type="submit"
            className="w-full bg-blue-600 text-white py-2 rounded hover:bg-blue-700 transition duration-200"
          >
            Verify
          </button>
        </form>

        <div className="mt-4 space-y-2">
          <button
            type="button"
            onClick={handleResendOTP}
            className="w-full bg-gray-600 text-white py-2 rounded hover:bg-gray-700 transition duration-200"
          >
            Resend OTP
          </button>

          {onOTPVerified && (
            <button
              type="button"
              onClick={onOTPVerified}
              className="w-full bg-green-600 text-white py-2 rounded hover:bg-green-700 transition duration-200"
            >
              Go to Login
            </button>
          )}

          {onBackToSignup && (
            <button
              type="button"
              onClick={onBackToSignup}
              className="w-full bg-gray-300 text-gray-700 py-2 rounded hover:bg-gray-400 transition duration-200"
            >
              Back to Signup
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default VerifyOTP;
