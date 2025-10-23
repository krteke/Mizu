"use client";

import { useState, useEffect } from "react";
import { HologramSticker } from "holographic-sticker";
import Image from "next/image";

type HolographicAvatarProps = {
  imageUrl: string;
  alt?: string;
  className?: string;
  textureUrl?: string;
  size?: number; // 头像尺寸
};

const HolographicAvatar = ({
  imageUrl,
  alt = "Holographic Avatar",
  className = "",
  textureUrl,
  size = 128,
}: HolographicAvatarProps) => {
  const [isMounted, setIsMounted] = useState(false);

  useEffect(() => {
    setIsMounted(true);
  }, []);

  return (
    <div
      className={className}
      style={{
        backfaceVisibility: "hidden",
        WebkitBackfaceVisibility: "hidden",
        width: size,
        height: size,
      }}
    >
      {isMounted ? (
        <HologramSticker.Root>
          <HologramSticker.Scene>
            <HologramSticker.Card
              className="relative rounded-full border-4 border-white dark:border-gray-800 brightness-125 dark:brightness-100"
              aspectRatio={1}
            >
              <HologramSticker.ImageLayer src={imageUrl} objectFit="contain" />
              <HologramSticker.Pattern
                opacity={0.5}
                maskUrl={imageUrl}
                maskSize="contain"
                textureUrl={textureUrl}
              >
                <HologramSticker.Refraction intensity={2} />
              </HologramSticker.Pattern>
            </HologramSticker.Card>
          </HologramSticker.Scene>
        </HologramSticker.Root>
      ) : (
        <Image
          src={imageUrl}
          alt={alt}
          width={size}
          height={size}
          className="h-full w-full object-cover"
          priority
        />
      )}
    </div>
  );
};

export default HolographicAvatar;
