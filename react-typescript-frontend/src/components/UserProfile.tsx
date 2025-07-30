// src/components/UserProfile.tsx
import { useState, useEffect } from "react";

interface UserProfileProps {
  token: string;
}

interface UserProfile {
  id: number;
  username: string;
  email: string;
  created_at: string;
  updated_at: string;
}

const UserProfile = ({ token }: UserProfileProps) => {
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const fetchProfile = async () => {
    setLoading(true);
    setError("");

    try {
      const response = await fetch("http://localhost:8080/api/me", {
        method: "GET",
        headers: {
          "Authorization": `Bearer ${token}`,
          "Content-Type": "application/json",
        },
      });

      const data = await response.json();

      if (response.ok) {
        setProfile(data);
      } else {
        setError(data.error || "Failed to fetch profile");
      }
    } catch (err) {
      setError("Something went wrong. Make sure backend is running.");
    }

    setLoading(false);
  };

  useEffect(() => {
    if (token) {
      fetchProfile();
    }
  }, [token]);

  if (loading) {
    return (
      <div className="bg-white p-6 rounded-lg shadow-md">
        <p className="text-center">Loading profile...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white p-6 rounded-lg shadow-md">
        <p className="text-red-600 text-center mb-4">{error}</p>
        <button
          onClick={fetchProfile}
          className="w-full bg-blue-600 text-white py-2 rounded hover:bg-blue-700 transition duration-200"
        >
          Retry
        </button>
      </div>
    );
  }

  if (!profile) {
    return (
      <div className="bg-white p-6 rounded-lg shadow-md">
        <p className="text-center">No profile data available</p>
      </div>
    );
  }

  return (
    <div className="bg-white p-6 rounded-lg shadow-md">
      <h3 className="text-xl font-bold mb-4">User Profile</h3>
      <div className="space-y-3">
        <div>
          <label className="block text-sm font-medium text-gray-700">User ID</label>
          <p className="text-gray-900">{profile.id}</p>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700">Username</label>
          <p className="text-gray-900">{profile.username}</p>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700">Email</label>
          <p className="text-gray-900">{profile.email}</p>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700">Created At</label>
          <p className="text-gray-900">{new Date(profile.created_at).toLocaleString()}</p>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700">Updated At</label>
          <p className="text-gray-900">{new Date(profile.updated_at).toLocaleString()}</p>
        </div>
      </div>
      <button
        onClick={fetchProfile}
        className="w-full mt-4 bg-green-600 text-white py-2 rounded hover:bg-green-700 transition duration-200"
      >
        Refresh Profile
      </button>
    </div>
  );
};

export default UserProfile;
