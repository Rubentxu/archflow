import { Server, Zap, Database, HardDrive, Globe, Lock } from "lucide-react";

const templates = [
  { type: "aws-ec2", name: "EC2 Instance", icon: Server },
  { type: "aws-lambda", name: "Lambda", icon: Zap },
  { type: "aws-rds", name: "RDS", icon: Database },
  { type: "aws-s3", name: "S3 Bucket", icon: HardDrive },
  { type: "api-gateway", name: "API Gateway", icon: Globe },
  { type: "vpc", name: "VPC", icon: Lock },
];

export default function Sidebar() {
  return (
    <aside className="w-64 bg-surface-dark/90 border-r border-border-dark flex flex-col">
      <div className="p-3 border-b border-border-dark">
        <h3 className="text-sm font-medium text-gray-300">Components</h3>
      </div>

      <div className="flex-1 overflow-y-auto p-2">
        <div className="space-y-1">
          {templates.map((template) => {
            const Icon = template.icon;
            return (
              <button
                key={template.type}
                draggable
                className="
                  w-full flex items-center gap-3 px-3 py-2 rounded
                  bg-white/5 hover:bg-white/10 transition-colors
                  text-left
                "
              >
                <Icon className="w-5 h-5 text-gray-400" />
                <span className="text-sm text-gray-200">{template.name}</span>
              </button>
            );
          })}
        </div>
      </div>
    </aside>
  );
}
