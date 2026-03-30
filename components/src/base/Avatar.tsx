import React from 'react';
import './Avatar.css';

export interface AvatarProps extends React.HTMLAttributes<HTMLDivElement> {
  src?: string;
  alt?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  variant?: 'circle' | 'square';
  fallback?: string;
}

export const Avatar: React.FC<AvatarProps> = ({
  src,
  alt = '',
  size = 'md',
  variant = 'circle',
  fallback,
  className = '',
  ...props
}) => {
  const [hasError, setHasError] = React.useState(false);

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map((n) => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  return (
    <div
      className={`avatar avatar-${size} avatar-${variant} ${className}`}
      {...props}
    >
      {src && !hasError ? (
        <img
          src={src}
          alt={alt}
          className="avatar-img"
          onError={() => setHasError(true)}
        />
      ) : (
        <span className="avatar-fallback">
          {fallback ? getInitials(fallback) : '?'}
        </span>
      )}
    </div>
  );
};
